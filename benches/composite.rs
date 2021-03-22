mod shared;

use {
    broot::{
        command::CommandParts,
        pattern::*,
    },
    criterion::{black_box, criterion_group, criterion_main, Criterion},
};

// this file benches composite patterns on file names so don't
// use file content sub patterns here
static PATTERNS: &[&str] = &[
    "r&!e",
];

fn score_of_composite_benchmark(c: &mut Criterion) {
    let search_modes = SearchModeMap::default();
    for pattern in PATTERNS {
        let task = format!("Pattern({:?})::score_of", &pattern);
        let parts = CommandParts::from(pattern.to_string());
        c.bench_function(&task, |b| {
            let cp = Pattern::new(&parts.pattern, &search_modes).unwrap();
            b.iter(|| {
                for name in shared::NAMES {
                    black_box(cp.score_of_string(name));
                }
            });
        });
    }
}

criterion_group!(
    name = composite;
    config = Criterion::default().without_plots();
    targets = score_of_composite_benchmark,
);
criterion_main!(composite);
