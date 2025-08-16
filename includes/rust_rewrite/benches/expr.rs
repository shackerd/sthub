use std::{hint::black_box, str::FromStr};

use criterion::{Criterion, criterion_group, criterion_main};
use mod_rewrite::{self, ExprGroup, Expression, Rewrite};
use pprof::criterion::{Output, PProfProfiler};

fn profiled() -> Criterion {
    let output = Output::Flamegraph(None);
    let prof = PProfProfiler::new(1000, output);
    Criterion::default().with_profiler(prof)
}

pub fn rewrite_match(g: &ExprGroup) {
    assert!(matches!(
        g.rewrite("/static/hello/world"),
        Ok(Rewrite::Uri(uri)) if uri == "/files/hello%2Fworld",
    ))
}

pub fn rewrite_match_ne(g: &ExprGroup) {
    assert!(matches!(
        g.rewrite("/static/hello/world"),
        Ok(Rewrite::Uri(uri)) if uri == "/files/hello/world",
    ))
}

pub fn bench_rule_match(c: &mut Criterion) {
    let e = Expression::from_str("RewriteRule /static/(.*) /files/$1").unwrap();
    let g = ExprGroup::new(vec![e]);
    c.bench_function("basic_match", |b| {
        b.iter(|| black_box(rewrite_match(black_box(&g))))
    });
}

pub fn bench_rule_match_ne(c: &mut Criterion) {
    let e = Expression::from_str("RewriteRule /static/(.*) /files/$1 [NE]").unwrap();
    let g = ExprGroup::new(vec![e]);
    c.bench_function("basic_match_ne", |b| {
        b.iter(|| black_box(rewrite_match_ne(black_box(&g))))
    });
}

criterion_group!(
    name = benches;
    config = profiled();
    targets = bench_rule_match, bench_rule_match_ne
);
criterion_main!(benches);
