use {
    broot::path,
    criterion::{black_box, criterion_group, criterion_main, Criterion},
};

static PATHS: &[&str] = &[
    "/abc/test/../thing.png",
    "/abc/def/../../thing.png",
    "/home/dys/test",
    "/home/dys",
    "/home/dys/",
    "/home/dys/..",
    "/home/dys/../",
    "/..",
    "../test",
    "/home/dys/../../../test",
    "/a/b/c/d/e/f/g/h/i/j/k/l/m/n",
    "/a/b/c/d/e/f/g/h/i/j/k/l/m/n/",
    "/",
    "π/2",
];

fn normalization_benchmark(c: &mut Criterion) {
    c.bench_function("normalize_path", |b| {
        b.iter(|| {
            for path in PATHS {
                black_box(path::normalize_path(path));
            }
        });
    });
}

criterion_group!(
    name = path_normalization;
    config = Criterion::default().without_plots();
    targets = normalization_benchmark,
);
criterion_main!(path_normalization);
