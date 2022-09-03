use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use anyhow::Result;
use clap::{AppSettings, Parser, Subcommand};
use evanescence_core::geometry::point::IPoint;
use evanescence_core::geometry::region::BoundingRegion;
use evanescence_core::geometry::storage::struct_of_arrays::IntoSoa;
use evanescence_core::geometry::storage::Soa;
use evanescence_core::numerics::monte_carlo::accept_reject::AcceptReject;
use evanescence_core::numerics::monte_carlo::MonteCarlo;
use evanescence_core::orbital::{AtomicReal, Qn};
use serde::Serialize;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
struct Cli {
    #[clap(subcommand)]
    command: Command,
    #[clap(short = 'n')]
    #[clap(value_parser = clap::value_parser!(u64).range(1..))]
    /// Number of points to sample
    count: u64,
    #[clap(short)]
    /// Path of output json, default stdout
    out: Option<PathBuf>,
    #[clap(long)]
    /// Whether to normalize wavefunction magnitudes to [0, 1]
    normalize_magnitudes: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[clap(allow_negative_numbers = true)]
    /// Sample real hydrogen orbitals
    AtomicReal { n: u32, l: u32, m: i32 },
}

fn sample_real<P: IPoint<3>, M: MonteCarlo<3, P, Output = f32>>(
    sampler: &mut M,
    count: usize,
    normalize_magnitudes: bool,
) -> Soa<3, f32> {
    let mut samples = sampler.take(count as _).into_soa();
    if normalize_magnitudes {
        let max = samples
            .values()
            .iter()
            .map(|p| p.abs())
            .max_by(f32::total_cmp)
            .unwrap();
        for v in samples.values_mut() {
            *v /= max;
        }
    }
    samples
}

fn write_json(out: Option<&PathBuf>, json: impl Serialize) -> Result<()> {
    let out_writer: Box<dyn Write> = if let Some(out) = out {
        Box::new(
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(out)?,
        )
    } else {
        Box::new(std::io::stdout())
    };
    let mut writer = BufWriter::new(out_writer);
    let write_json = if out.is_some() {
        serde_json::to_writer
    } else {
        serde_json::to_writer_pretty
    };
    write_json(&mut writer, &json)?;
    writer.flush()?;
    Ok(())
}

#[derive(Serialize)]
struct AtomicRealJson {
    qn: Qn,
    bounding_sphere_radius: f32,
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
    psi: Vec<f32>,
}
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::AtomicReal { n, l, m } => {
            let qn = Qn::new(n, l, m)?;
            let atomic_real = AtomicReal::new(qn);
            let bounding_sphere_radius = atomic_real.bounding_region().radius;

            let sampler = &mut AcceptReject::new(atomic_real);
            let ([x, y, z], psi) =
                sample_real(sampler, cli.count as _, cli.normalize_magnitudes).into_components();

            let json = AtomicRealJson {
                qn,
                bounding_sphere_radius,
                x,
                y,
                z,
                psi,
            };
            write_json(cli.out.as_ref(), json)?;
        }
    }

    Ok(())
}
