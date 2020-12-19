use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use evanescence_core::numerics::{
    orthogonal_polynomials::associated_laguerre, orthogonal_polynomials::associated_legendre,
    Factorial, Multifactorial,
};
use evanescence_core::orbital::{
    monte_carlo::{MonteCarlo, Quality},
    QuantumNumbers, RealOrbital,
};

pub fn bench_numerics(c: &mut Criterion) {
    let mut fact_group = c.benchmark_group("factorials");
    for n in [3_u64, 7, 12].iter() {
        fact_group.bench_with_input(BenchmarkId::new("factorial", n), n, |b, n| {
            b.iter(|| n.factorial())
        });

        fact_group.bench_with_input(BenchmarkId::new("multifactorial::<2>", n), n, |b, n| {
            b.iter(|| n.multifactorial::<2>())
        });
    }
    fact_group.finish();

    let mut poly_group = c.benchmark_group("orthogonal_polynomials");
    for params in [(1_u32, 0_u32), (3, 2), (4, 4), (6, 1)].iter() {
        poly_group.bench_with_input(
            BenchmarkId::new("associated_legendre", format!("{},{}", params.0, params.1)),
            params,
            |b, lm| b.iter(|| associated_legendre(*lm, 27.1828)),
        );

        poly_group.bench_with_input(
            BenchmarkId::new("associated_laguerre", format!("{},{}", params.0, params.1)),
            params,
            |b, lm| b.iter(|| associated_laguerre(*lm, 27.1828)),
        );
    }
    poly_group.finish();
}

macro_rules! bench_orbitals {
    ($group:ident, $quality:expr, $(($n: literal $l:literal $m:literal)),*) => {
        $group.throughput(Throughput::Elements($quality as _));
        $(
            let qn = QuantumNumbers::new($n, $l, $m).unwrap();
            $group.bench_function(
                format!("{}", qn),
                |b| b.iter(|| RealOrbital::monte_carlo_simulate(qn, $quality))
            );
        )*
    }
}

pub fn bench_monte_carlo(c: &mut Criterion) {
    let mut group = c.benchmark_group("monte_carlo");
    group
        .sample_size(10)
        .warm_up_time(Duration::from_secs(10))
        .measurement_time(Duration::from_secs(30));
    bench_orbitals!(
        group,
        Quality::Extreme,
        (1 0  0),
        (2 0  0),
        (2 1 -1),
        (3 2 -2),
        (4 1  1),
        (4 2  0),
        (4 3 -1),
        (5 0  0),
        (5 4  4)
    );
}

criterion_group!(benches, bench_numerics, bench_monte_carlo);
criterion_main!(benches);
