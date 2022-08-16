#![feature(once_cell)]

use std::f32::consts::{FRAC_1_SQRT_2, SQRT_2};
use std::iter;
use std::sync::LazyLock;
use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use evanescence_core::geometry::point::SphericalPoint3;
use evanescence_core::geometry::region::{
    BallCenteredAtOrigin, BoundingRegion, CubeCenteredAtOrigin, Region,
};
use evanescence_core::lc;
use evanescence_core::numerics::monte_carlo::accept_reject::AcceptReject;
use evanescence_core::numerics::polynomial::Polynomial;
use evanescence_core::numerics::random::WyRand;
use evanescence_core::numerics::{normalize, Function};
use evanescence_core::orbital::hybrid::Hybrid;
use evanescence_core::orbital::{AtomicComplex, AtomicReal, Qn};
use nalgebra::Point3;

fn polynomial(crit: &mut Criterion) {
    let mut group = crit.benchmark_group("polynomial");
    group.throughput(Throughput::Elements(1));

    let rng = &mut WyRand::new_seed(0xDEAD_BEEF);

    for n in 1..=10 {
        let poly: Polynomial = iter::repeat_with(|| rng.gen_f32())
            .take(n + 1)
            .map(|a_i| normalize(0.0..=1.0, -10.0..=10.0, a_i))
            .collect();

        group.bench_function(format!("horner_deg{n}"), |b| {
            b.iter(|| {
                black_box(poly.evaluate_horner(std::f32::consts::TAU));
            })
        });
    }

    group.finish();
}

fn sampling(crit: &mut Criterion) {
    let mut group = crit.benchmark_group("sampling");
    group.throughput(Throughput::Elements(1));

    group.bench_function("ball/spherical_point_3", |b| {
        let ball = BallCenteredAtOrigin { radius: 20. };
        let rng = &mut WyRand::new();
        b.iter(|| black_box(Region::<3, SphericalPoint3>::sample(&ball, rng)));
    });

    group.bench_function("ball/point_3", |b| {
        let ball = BallCenteredAtOrigin { radius: 20. };
        let rng = &mut WyRand::new();
        b.iter(|| black_box(Region::<3, Point3<_>>::sample(&ball, rng)));
    });

    group.bench_function("cube/point_3", |b| {
        let cube = CubeCenteredAtOrigin { side_length: 40. };
        let rng = &mut WyRand::new();
        b.iter(|| black_box(Region::<3, Point3<_>>::sample(&cube, rng)));
    });

    group.finish();
}

static QNS: LazyLock<Vec<(usize, Qn)>> = LazyLock::new(|| {
    Qn::enumerate_up_to_n(5)
        .unwrap()
        .enumerate()
        // .filter(|(_, qn)| [1, 2, 5].contains(&qn.n()))
        .collect()
});

fn atomic_real(crit: &mut Criterion) {
    let mut group = crit.benchmark_group("atomic_real_evaluation");
    group.throughput(Throughput::Elements(1));
    for (idx, qn) in QNS.iter() {
        group.bench_function(format!("{idx:02}_{qn}"), |b| {
            let real = AtomicReal::new(*qn);
            let pt = real.bounding_region().sample(&mut WyRand::new());
            b.iter(|| black_box(real.evaluate(&pt)));
        });
    }
    group.finish();

    let mut group = crit.benchmark_group("atomic_real_monte_carlo_startup");
    for (idx, qn) in QNS.iter() {
        group.bench_function(format!("{idx:02}_{qn}"), |b| {
            b.iter(|| black_box(AcceptReject::new(AtomicReal::new(*qn))));
        });
    }
    group.finish();

    let mut group = crit.benchmark_group("atomic_real_monte_carlo_sampling");
    group.throughput(Throughput::Elements(1));
    for (idx, qn) in QNS.iter() {
        group.bench_function(format!("{idx:02}_{qn}"), |b| {
            let mut sampler = AcceptReject::new(AtomicReal::new(*qn));
            b.iter(|| black_box(sampler.next()));
        });
    }
    group.finish();
}

fn atomic_complex(crit: &mut Criterion) {
    let mut group = crit.benchmark_group("atomic_complex_monte_carlo_sampling");
    group.throughput(Throughput::Elements(1));
    for (idx, qn) in QNS.iter() {
        group.bench_function(format!("{idx:02}_{qn}"), |b| {
            let mut sampler = AcceptReject::new(AtomicComplex::new(*qn));
            b.iter(|| black_box(sampler.next()));
        });
    }
    group.finish();
}

fn hybrid(crit: &mut Criterion) {
    let mut group = crit.benchmark_group("hybrid_monte_carlo_sampling");
    group.throughput(Throughput::Elements(1));

    group.bench_function("sp2", |b| {
        let sp2 = lc! {
            overall: 0.408_248_3,
            (2, 0, 0) * SQRT_2,
            (2, 1, 1) * 1.0,
            (2, 1, -1) * 1.732_050_8,
        };
        let mut sampler = AcceptReject::new(Hybrid::new(sp2));
        b.iter(|| black_box(sampler.next()));
    });

    group.bench_function("sp3d2", |b| {
        let sp3d2 = lc! {
            overall: 0.408_248_3,
            (3, 0, 0) * 1.0,
            (3, 1, 1) * 1.732_050_8,
            (3, 2, 0) * -FRAC_1_SQRT_2,
            (3, 2, 2) * 1.732_050_8 * FRAC_1_SQRT_2,
        };
        let mut sampler = AcceptReject::new(Hybrid::new(sp3d2));
        b.iter(|| black_box(sampler.next()));
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(2))
        .measurement_time(Duration::from_secs(4));
    targets = polynomial, sampling, atomic_real, atomic_complex, hybrid
}
criterion_main!(benches);
