use crate::package_tree;
use std::ffi::OsString;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub mod emojis {
    use console::Emoji;
    pub static TREE: Emoji<'_, '_> = Emoji("🌴 ", "");
    pub static SWEEP: Emoji<'_, '_> = Emoji("🧹 ", "");
    pub static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍 ", "");
    pub static CODE: Emoji<'_, '_> = Emoji("🟰  ", "");
    pub static SWORDS: Emoji<'_, '_> = Emoji("⚔️  ", "");
    pub static DEPS: Emoji<'_, '_> = Emoji("️🕸️  ", "");
    pub static CHECKMARK: Emoji<'_, '_> = Emoji("️✅  ", "");
    pub static CROSS: Emoji<'_, '_> = Emoji("️🛑  ", "");
    pub static LINE_CLEAR: &str = "\x1b[2K";
}

pub trait LexicalAbsolute {
    fn to_lexical_absolute(&self) -> std::io::Result<PathBuf>;
}

impl LexicalAbsolute for Path {
    fn to_lexical_absolute(&self) -> std::io::Result<PathBuf> {
        let mut absolute = if self.is_absolute() {
            PathBuf::new()
        } else {
            std::env::current_dir()?
        };
        for component in self.components() {
            match component {
                Component::CurDir => {}
                Component::ParentDir => {
                    absolute.pop();
                }
                component @ _ => absolute.push(component.as_os_str()),
            }
        }
        Ok(absolute)
    }
}

pub fn get_package_path(root: &str, package_name: &str) -> String {
    format!("{}/node_modules/{}", root, package_name)
}

pub fn get_build_path(root: &str, package_name: &str) -> String {
    format!("{}/node_modules/{}/lib/ocaml", root, package_name)
}

pub fn get_bs_build_path(root: &str, package_name: &str) -> String {
    format!("{}/node_modules/{}/lib/bs", root, package_name)
}

pub fn get_path(root: &str, package_name: &str, file: &str) -> String {
    format!("{}/{}/{}", root, package_name, file)
}

pub fn get_node_modules_path(root: &str) -> String {
    format!("{}/node_modules", root)
}

pub fn get_abs_path(path: &str) -> String {
    let abs_path_buf = PathBuf::from(path);

    return abs_path_buf
        .to_lexical_absolute()
        .expect("Could not canonicalize")
        .to_str()
        .expect("Could not canonicalize")
        .to_string();
}

pub fn get_basename(path: &str) -> String {
    let path_buf = PathBuf::from(path);
    return path_buf
        .file_stem()
        .expect("Could not get basename")
        .to_str()
        .expect("Could not get basename")
        .to_string();
}

pub fn change_extension(path: &str, new_extension: &str) -> String {
    let path_buf = PathBuf::from(path);
    return path_buf
        .with_extension(new_extension)
        .to_str()
        .expect("Could not change extension")
        .to_string();
}

/// Capitalizes the first character in s.
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn add_suffix(base: &str, namespace: &package_tree::Namespace) -> String {
    match namespace {
        package_tree::Namespace::NamespaceWithEntry {
            namespace: _,
            entry,
        } if entry == base => base.to_string(),
        package_tree::Namespace::Namespace(namespace)
        | package_tree::Namespace::NamespaceWithEntry {
            namespace,
            entry: _,
        } => base.to_string() + "-" + &namespace,
        package_tree::Namespace::NoNamespace => base.to_string(),
    }
}

pub fn module_name_with_namespace(
    module_name: &str,
    namespace: &package_tree::Namespace,
) -> String {
    capitalize(&add_suffix(module_name, namespace))
}

// this doesn't capitalize the module name! if the rescript name of the file is "foo.res" the
// compiler assets are foo-Namespace.cmt and foo-Namespace.cmj, but the module name is Foo
pub fn file_path_to_compiler_asset_basename(
    path: &str,
    namespace: &package_tree::Namespace,
) -> String {
    let base = get_basename(path);
    add_suffix(&base, namespace)
}

pub fn file_path_to_module_name(path: &str, namespace: &package_tree::Namespace) -> String {
    capitalize(&file_path_to_compiler_asset_basename(path, namespace))
}

pub fn contains_ascii_characters(str: &str) -> bool {
    for chr in str.chars() {
        if chr.is_ascii_alphanumeric() {
            return true;
        }
    }
    return false;
}

pub fn create_build_path(build_path: &str) {
    fs::DirBuilder::new()
        .recursive(true)
        .create(PathBuf::from(build_path.to_string()))
        .unwrap();
}

pub fn get_bsc(root_path: &str) -> String {
    let subfolder = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "darwinarm64",
        ("macos", _) => "darwin",
        ("linux", _) => "linux",
        ("windows", _) => "win32",
        _ => panic!("Unsupported architecture"),
    };

    get_node_modules_path(root_path) + "/rescript/" + subfolder + "/bsc.exe"
}

pub fn string_ends_with_any(s: &PathBuf, suffixes: &[&str]) -> bool {
    suffixes.iter().any(|&suffix| {
        s.extension()
            .unwrap_or(&OsString::new())
            .to_str()
            .unwrap_or("")
            == suffix
    })
}

pub fn get_compiler_asset(
    source_file: &str,
    package_name: &str,
    namespace: &package_tree::Namespace,
    root_path: &str,
    extension: &str,
) -> String {
    get_build_path(root_path, package_name)
        + "/"
        + &file_path_to_compiler_asset_basename(source_file, namespace)
        + "."
        + extension
}

pub fn canonicalize_string_path(path: &str) -> Option<String> {
    return Path::new(path)
        .canonicalize()
        .ok()
        .map(|path| path.to_str().expect("Could not canonicalize").to_string());
}

// sometimes we only want to canonicalize the parent path, if we potentially want to
// canonicalize file paths that might not exist anymore BUT the parent path does
pub fn canonicalize_parent_string_path(path: &str) -> Option<String> {
    return Path::new(path)
        .parent()
        .unwrap()
        .canonicalize()
        .ok()
        .map(|dir| {
            let filename = Path::new(path)
                .file_name()
                .expect("There should always be a filename");
            // add back file
            let path = dir.join(filename);
            return path.to_str().expect("Could not canonicalize").to_string();
        });
}

pub fn get_bs_compiler_asset(
    source_file: &str,
    package_name: &str,
    namespace: &package_tree::Namespace,
    root_path: &str,
    extension: &str,
) -> String {
    let namespace = match extension {
        "ast" | "iast" => &package_tree::Namespace::NoNamespace,
        _ => namespace,
    };
    let canonicalized_source_file = source_file;
    let canonicalized_path =
        canonicalize_string_path(&get_package_path(root_path, &package_name)).unwrap();

    let dir = std::path::Path::new(&canonicalized_source_file)
        .strip_prefix(canonicalized_path)
        .unwrap()
        .parent()
        .unwrap();

    std::path::Path::new(&get_bs_build_path(root_path, &package_name))
        .join(dir)
        .join(file_path_to_compiler_asset_basename(source_file, namespace) + extension)
        .to_str()
        .unwrap()
        .to_owned()
}

pub fn get_namespace_from_module_name(module_name: &str) -> Option<String> {
    let mut split = module_name.split("-");
    let _ = split.next();
    split.next().map(|s| s.to_string())
}

pub fn is_interface_ast_file(file: &str) -> bool {
    file.ends_with(".iast")
}

pub fn get_mlmap_path(root_path: &str, package_name: &str, namespace: &str) -> String {
    get_build_path(root_path, package_name) + "/" + namespace + ".mlmap"
}

pub fn get_mlmap_compile_path(root_path: &str, package_name: &str, namespace: &str) -> String {
    get_build_path(root_path, package_name) + "/" + namespace + ".cmi"
}

pub fn get_ast_path(source_file: &str, package_name: &str, root_path: &str) -> String {
    get_compiler_asset(
        source_file,
        package_name,
        &package_tree::Namespace::NoNamespace,
        root_path,
        "ast",
    )
}

pub fn get_iast_path(source_file: &str, package_name: &str, root_path: &str) -> String {
    get_compiler_asset(
        source_file,
        package_name,
        &package_tree::Namespace::NoNamespace,
        root_path,
        "iast",
    )
}

pub fn read_lines(filename: String) -> io::Result<io::Lines<io::BufReader<fs::File>>> {
    let file = fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn get_system_time() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

pub fn is_interface_file(extension: &str) -> bool {
    match extension {
        "resi" | "mli" | "rei" => true,
        _ => false,
    }
}

pub fn is_implementation_file(extension: &str) -> bool {
    match extension {
        "res" | "ml" | "re" => true,
        _ => false,
    }
}

pub fn is_source_file(extension: &str) -> bool {
    is_interface_file(extension) || is_implementation_file(extension)
}
