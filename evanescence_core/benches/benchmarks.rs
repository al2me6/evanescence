use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use evanescence_core::geometry::storage::Soa;
use evanescence_core::numerics::monte_carlo::accept_reject::AcceptReject;
use evanescence_core::numerics::monte_carlo::MonteCarlo;
use evanescence_core::numerics::special::factorial::DoubleFactorial;
use evanescence_core::numerics::special::orthogonal_polynomials::{
    associated_laguerre, renormalized_associated_legendre,
};
use evanescence_core::orbital::{Qn, Real};

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
            |b, lm| b.iter(|| renormalized_associated_legendre(*lm, 27.1828)),
        );

        poly_group.bench_with_input(
            BenchmarkId::new("associated_laguerre", format!("{}_{}", params.0, params.1)),
            &params,
            |b, lm| {
                let evaluator = associated_laguerre(lm.0, lm.1);
                b.iter(|| evaluator.evaluate_horner(27.1828));
            },
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
    group.throughput(Throughput::Elements(131_072));
    for qn in Qn::enumerate_up_to_n(5).unwrap() {
        group.bench_function(
            BenchmarkId::new(
                "real",
                format!(
                    "{}_{}_{}",
                    qn.n(),
                    qn.l(),
                    qn.m().to_string().replace('-', "n")
                ),
            ),
            |b| {
                b.iter(|| {
                    AcceptReject::new(Real::new(qn))
                        .simulate(131_072)
                        .into_iter()
                        .collect::<Soa<3, _>>()
                })
            },
        );
    }
}

criterion_group!(benches, bench_numerics, bench_monte_carlo);
criterion_main!(benches);
