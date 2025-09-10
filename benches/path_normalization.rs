use {
    broot::path,
    glassbench::*,
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

fn bench_normalization(gb: &mut Bench) {
    gb.task("normalize_path", |b| {
        b.iter(|| {
            for path in PATHS {
                pretend_used(path::normalize_path(path));
            }
        });
    });
}

glassbench!(
    "Path Normalization",
    bench_normalization,
);
