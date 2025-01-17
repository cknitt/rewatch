use criterion::{criterion_group, criterion_main, Criterion};
use rewatch::build;
use rewatch::package_tree;
use rewatch::helpers;

use std::fs::File;
use std::io::prelude::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("build-package-tree", |b| {
        // Folder for the testrepo
        let folder = "walnut_monorepo";
        let project_root = helpers::get_abs_path(folder);

        b.iter(|| {
            package_tree::make(&project_root);
        })
    });

    c.bench_function("clean-build-change-build", |b| {
        // Folder for the testrepo
        let folder = "testrepo";
        let filename = "testrepo/packages/dep02/src/Dep02.res";
        // Clean the build
        build::clean(folder);
        // Read the file we'll be mutating
        let mut file = File::options()
            .read(true)
            .write(true)
            .append(true)
            .open(filename)
            .unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        b.iter(|| {
            // Create initial build
            let _ = build::build(folder);
            // Update the file
            let _ = writeln!(
                file,
                r#"let log2 = () => ["a", "b"]->forEach(Js.log);log2()"#
            );
            // Create another build
            let _ = build::build(folder);

            // Reset state
            File::create(filename).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
            let _ = build::build(folder);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
