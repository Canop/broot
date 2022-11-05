mod shared;

use {
    broot::{
        command::CommandParts,
        pattern::*,
    },
    glassbench::*,
};

// this file benches composite patterns on file names so don't
// use file content sub patterns here
static PATTERNS: &[&str] = &[
    "réveil",
    "r&!e",
    "(!e&!b)|c",
];

fn bench_score_of_composite(gb: &mut Bench) {
    let search_modes = SearchModeMap::default();
    for pattern in PATTERNS {
        let name = format!("Composite({:?})::score_of", &pattern);
        gb.task(name, |b| {
            let parts = CommandParts::from(pattern.to_string());
            let cp = Pattern::new(&parts.pattern, &search_modes, 10*1024*1024).unwrap();
            b.iter(|| {
                for name in shared::NAMES {
                    pretend_used(cp.score_of_string(name));
                }
            });
        });
    }
}

glassbench!(
    "Composite Patterns",
    bench_score_of_composite,
);
