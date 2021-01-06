use std::convert::TryInto;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use argh::FromArgs;
use evanescence_core::monte_carlo::{MonteCarlo, Quality};
use evanescence_core::orbital::{self, Orbital, QuantumNumbers, RadialPlot};
use pyo3::prelude::*;
use pyo3::types::PyModule;
use strum::{Display, EnumString};

#[derive(Clone, Copy, Display, EnumString)]
enum Mode {
    Pointillist,
    Radial,
    RadialProbability,
    RadialProbabilityDensity,
}

impl TryInto<orbital::RadialPlot> for Mode {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<orbital::RadialPlot, Self::Error> {
        match self {
            Self::Radial => Ok(RadialPlot::Wavefunction),
            Self::RadialProbability => Ok(RadialPlot::Probability),
            Self::RadialProbabilityDensity => Ok(RadialPlot::ProbabilityDensity),
            _ => Err(anyhow!("Cannot plot {} as a radial plot.", self)),
        }
    }
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
    /// select the visualization computed: Pointillist (default), Radial, RadialProbability,
    /// RadialProbabilityDensity,
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
        Mode::Radial | Mode::RadialProbability | Mode::RadialProbabilityDensity => {
            run_simulation(
                || {
                    let num_points = quality as usize / 100;
                    (
                        num_points,
                        orbital::Real::plot_radial(qn, mode.try_into().unwrap(), num_points),
                    )
                },
                skip_render,
                |sim_result| {
                    renderer.call1("render_1d", sim_result)?;
                    Ok(())
                },
            )?;
        }
    }

    Ok(())
}
