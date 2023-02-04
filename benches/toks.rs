mod shared;

use {
    broot::pattern::TokPattern,
    glassbench::*,
};

static PATTERNS: &[&str] = &["a", "réveil", "bro,c", "e,jenc,arec,ehro", "broot"];

fn bench_score_of_toks(gb: &mut Bench) {
    for pattern in PATTERNS {
        let task_name = format!("TokPattern({pattern:?})::score_of");
        gb.task(task_name, |b| {
            let fp = TokPattern::new(pattern);
            b.iter(|| {
                for name in shared::NAMES {
                    pretend_used(fp.score_of(name));
                }
            });
        });
    }
}

glassbench!(
    "Tokens Patterns",
    bench_score_of_toks,
);
