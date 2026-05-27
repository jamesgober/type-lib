//! Microbenchmarks for the validation hot path.
//!
//! These measure the cost of constructing a `Refined` value (a single
//! validation) for representative built-in rules and a composed combinator, plus
//! the zero-cost accessors. Run with `cargo bench`.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use type_lib::combinator::And;
use type_lib::rules::{Ascii, InRange, LenRange, NonEmpty};
use type_lib::Refined;

type Username = Refined<&'static str, And<NonEmpty, LenRange<3, 16>>>;
type Percent = Refined<i32, InRange<0, 100>>;
type AsciiStr = Refined<&'static str, Ascii>;

fn bench_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("refined_new");

    group.bench_function("len_range_str", |b| {
        b.iter(|| Username::new(black_box("alice")));
    });

    group.bench_function("in_range_i32", |b| {
        b.iter(|| Percent::new(black_box(50)));
    });

    group.bench_function("ascii_str", |b| {
        b.iter(|| AsciiStr::new(black_box("plain-text")));
    });

    group.finish();
}

fn bench_accessors(c: &mut Criterion) {
    let value = Percent::new(42).expect("42 is in range");

    let mut group = c.benchmark_group("refined_access");

    group.bench_function("get", |b| {
        b.iter(|| *black_box(&value).get());
    });

    group.bench_function("deref", |b| {
        b.iter(|| *black_box(&value));
    });

    group.finish();
}

criterion_group!(benches, bench_construction, bench_accessors);
criterion_main!(benches);
