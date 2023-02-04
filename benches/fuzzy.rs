mod shared;

use {
    broot::pattern::FuzzyPattern,
    glassbench::*,
};

static PATTERNS: &[&str] = &["réveil", "AB", "e", "brt", "brootz"];

fn bench_score_of_fuzzy(gb: &mut Bench) {
    for pattern in PATTERNS {
        let task_name = format!("Fuzzy({pattern:?})::score_of");
        gb.task(task_name, |b| {
            let fp = FuzzyPattern::from(pattern);
            b.iter(|| {
                for name in shared::NAMES {
                    pretend_used(fp.score_of(name));
                }
            });
        });
    }
}

glassbench!(
    "Fuzzy Patterns",
    bench_score_of_fuzzy,
);
