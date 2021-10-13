use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use evanescence_core::monte_carlo::{MonteCarlo, Quality};
use evanescence_core::numerics::orthogonal_polynomials::{
    associated_laguerre, associated_legendre,
};
use evanescence_core::numerics::DoubleFactorial;
use evanescence_core::orbital::{Qn, Real1};

pub fn bench_numerics(c: &mut Criterion) {
    let mut fact_group = c.benchmark_group("factorials");
    for n in 0_u64..=14 {
        fact_group.bench_with_input(BenchmarkId::new("multifactorial_2", n), &n, |b, n| {
            b.iter(|| n.double_factorial())
        });
    }
    fact_group.finish();

    let mut poly_group = c.benchmark_group("orthogonal_polynomials");
    for params in (1..=6).flat_map(|a| (0..=a).map(move |b| (a, b))) {
        poly_group.bench_with_input(
            BenchmarkId::new("associated_legendre", format!("{}_{}", params.0, params.1)),
            &params,
            |b, lm| b.iter(|| associated_legendre(*lm, 27.1828)),
        );

        poly_group.bench_with_input(
            BenchmarkId::new("associated_laguerre", format!("{}_{}", params.0, params.1)),
            &params,
            |b, lm| b.iter(|| associated_laguerre(*lm, 27.1828)),
        );
    }
    poly_group.finish();
}

pub fn bench_monte_carlo(c: &mut Criterion) {
    let mut group = c.benchmark_group("monte_carlo");
    group
        .sample_size(10)
        .warm_up_time(Duration::from_secs(10))
        .measurement_time(Duration::from_secs(30));
    group.throughput(Throughput::Elements(Quality::Extreme as _));
    for qn in Qn::enumerate_up_to_n(5).unwrap() {
        group.bench_with_input(
            BenchmarkId::new(
                "real",
                format!(
                    "{}_{}_{}",
                    qn.n(),
                    qn.l(),
                    qn.m().to_string().replace("-", "n")
                ),
            ),
            &qn,
            |b, qn| b.iter(|| Real1::monte_carlo_simulate(qn, Quality::Extreme, true)),
        );
    }
}

criterion_group!(benches, bench_numerics, bench_monte_carlo);
criterion_main!(benches);
