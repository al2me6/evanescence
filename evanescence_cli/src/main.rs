use std::time::Instant;

use anyhow::{Context, Result};
use argh::FromArgs;
use evanescence_core::geometry::Vec3;
use evanescence_core::monte_carlo::{MonteCarlo, Quality};
use evanescence_core::numerics::{ComponentForm, Evaluate};
use evanescence_core::orbital::{self, Orbital, QuantumNumbers, NL};
use pyo3::prelude::*;
use pyo3::types::PyModule;
use strum::{Display, EnumString};

#[derive(Display, EnumString)]
enum Mode {
    Pointillist,
    RadialPDF,
}

#[derive(FromArgs)]
/// Simple CLI for evanescence_core, using the Plotly Python library for plotting.
/// Note: pass two dashes before arguments for negative values: `evanescence_cli -- 4 2 -1`.
struct Args {
    #[argh(positional)]
    n: u32,
    #[argh(positional)]
    l: u32,
    #[argh(positional)]
    m: i32,
    #[argh(option, short = 'm', default = "Mode::Pointillist")]
    /// select the visualization computed: Pointillist (default), RadialPDF,
    mode: Mode,
    #[argh(option, short = 'q', default = "Quality::High")]
    /// render quality: Minimum, Low, Medium, High (default), VeryHigh, or Extreme
    quality: Quality,
    #[argh(switch)]
    /// skip rendering (effectively a benchmark for computation speed)
    skip_render: bool,
}

fn run_simulation<P, R, T>(prepare: P, skip_render: bool, render: R) -> Result<()>
where
    P: Fn() -> (usize, T),
    R: Fn(T) -> Result<()>,
{
    let now = Instant::now();
    let (num_points, sim_result) = prepare();
    println!(
        "Computed {} points in {:.3}s.",
        num_points,
        now.elapsed().as_secs_f64()
    );

    if skip_render {
        println!("Skipping rendering.");
        return Ok(());
    }

    let now = Instant::now();
    render(sim_result)?;
    println!("Rendered in {:.3}s.", now.elapsed().as_secs_f64());
    Ok(())
}

fn main() -> Result<()> {
    let Args {
        n,
        l,
        m,
        mode,
        quality,
        skip_render,
    } = argh::from_env();

    let qn = QuantumNumbers::new(n, l, m).with_context(|| {
        format!(
            "received illegal quantum numbers: n={}, l={}, m={}; must satisfy n > l and l >= |m|",
            n, l, m
        )
    })?;

    println!(
        "Rendering {} visualization for real orbital {} at {} quality...",
        mode, qn, quality
    );

    let gil = Python::acquire_gil();
    let py = gil.python();
    let renderer = PyModule::from_code(
        py,
        include_str!("renderers.py"),
        "renderers.py",
        "renderers",
    )?;

    match mode {
        Mode::Pointillist => {
            run_simulation(
                || {
                    (
                        quality as usize,
                        orbital::Real::monte_carlo_simulate(qn, quality),
                    )
                },
                skip_render,
                |sim_result| {
                    renderer.call1("render_pointillist", sim_result.into_components())?;
                    Ok(())
                },
            )?;
        }
        Mode::RadialPDF => {
            run_simulation(
                || {
                    let num_points = quality as usize / 100;
                    let nl: NL = qn.into();
                    let sim_result: ComponentForm<_> =
                        orbital::wavefunctions::RadialProbabilityDensity::evaluate_on_line_segment(
                            nl,
                            Vec3::ZERO,
                            Vec3::I * orbital::Real::estimate_radius(qn),
                            num_points,
                        )
                        .into();
                    let (xs, _, _, vals) = sim_result.into_components();
                    (num_points, (xs, vals))
                },
                skip_render,
                |(xs, vals)| {
                    renderer.call1("render_radial_pdf", (xs, vals))?;
                    Ok(())
                },
            )?;
        }
    }

    Ok(())
}
