use std::convert::TryInto;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use argh::FromArgs;
use evanescence_core::geometry::Plane;
use evanescence_core::monte_carlo::{MonteCarlo, Quality};
use evanescence_core::orbital::{self, Orbital, Qn, RadialPlot};
use pyo3::prelude::*;
use pyo3::types::PyModule;
use strum::{Display, EnumString};

#[derive(Clone, Copy, Display, EnumString)]
enum Mode {
    Pointillist,
    PointillistWithNodes,
    PointillistComplex,
    Radial,
    RadialProbability,
    RadialProbabilityDistribution,
    CrossSectionXY,
    CrossSectionYZ,
    CrossSectionZX,
}

impl TryInto<RadialPlot> for Mode {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<RadialPlot, Self::Error> {
        match self {
            Self::Radial => Ok(RadialPlot::Wavefunction),
            Self::RadialProbability => Ok(RadialPlot::Probability),
            Self::RadialProbabilityDistribution => Ok(RadialPlot::ProbabilityDistribution),
            _ => Err(anyhow!("Cannot plot {} as a radial plot.", self)),
        }
    }
}

impl TryInto<Plane> for Mode {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Plane, Self::Error> {
        match self {
            Self::CrossSectionXY => Ok(Plane::XY),
            Self::CrossSectionYZ => Ok(Plane::YZ),
            Self::CrossSectionZX => Ok(Plane::ZX),
            _ => Err(anyhow!("Cannot plot {} as a cross-section plot.", self)),
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
    /// select the visualization computed: Pointillist (default), PointillistWithNodes,
    /// PointillistComplex, Radial, RadialProbability, RadialProbabilityDensity, CrossSectionXY,
    /// CrossSectionYZ, CrossSectionZX,
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

    let qn = Qn::new(n, l, m).with_context(|| {
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
        Mode::PointillistWithNodes => {
            run_simulation(
                || {
                    let num_points_iso = quality.for_isosurface();
                    (
                        // We render the Monte Carlo points and a cube of side length num_points_iso.
                        quality as usize + num_points_iso.pow(3),
                        (
                            orbital::Real::monte_carlo_simulate(qn, quality),
                            orbital::Real::sample_region(qn, num_points_iso),
                        ),
                    )
                },
                skip_render,
                |(pointillist, isosurface)| {
                    let (xs_pt, ys_pt, zs_pt, vals_pt) = pointillist.into_components();
                    let (xs_iso, ys_iso, zs_iso, vals_iso) = isosurface.into_components();
                    let mut min = 0_f32;
                    let mut max = 0_f32;
                    vals_iso.iter().for_each(|&val| {
                        min = min.min(val);
                        max = max.max(val);
                    });
                    renderer.call1(
                        "render_pointillist_with_nodes",
                        (
                            xs_pt, ys_pt, zs_pt, vals_pt, xs_iso, ys_iso, zs_iso, vals_iso,
                        ),
                    )?;
                    Ok(())
                },
            )?;
        }
        Mode::PointillistComplex => {
            run_simulation(
                || {
                    (
                        quality as usize,
                        orbital::Complex::monte_carlo_simulate(qn, quality),
                    )
                },
                skip_render,
                |sim_result| {
                    let (xs, ys, zs, vals) = sim_result.into_components();
                    let vals_moduli: Vec<_> = vals.iter().map(|val| val.norm()).collect();
                    let vals_arguments: Vec<_> = vals.iter().map(|val| val.arg()).collect();
                    renderer.call1(
                        "render_pointillist_complex",
                        (xs, ys, zs, vals_moduli, vals_arguments),
                    )?;
                    Ok(())
                },
            )?;
        }
        Mode::Radial | Mode::RadialProbability | Mode::RadialProbabilityDistribution => {
            run_simulation(
                || {
                    let num_points = quality.for_line();
                    (
                        num_points,
                        orbital::Real::sample_radial(qn, mode.try_into().unwrap(), num_points),
                    )
                },
                skip_render,
                |(xs, ys)| {
                    renderer.call1("render_1d", (xs, "r", ys, mode.to_string()))?;
                    Ok(())
                },
            )?;
        }
        Mode::CrossSectionXY | Mode::CrossSectionYZ | Mode::CrossSectionZX => {
            run_simulation(
                || {
                    let num_points = quality.for_grid();
                    (
                        num_points * num_points, // We calculate an entire grid.
                        orbital::Real::sample_cross_section(
                            qn,
                            mode.try_into().unwrap(),
                            num_points,
                        ),
                    )
                },
                skip_render,
                |sim_result| {
                    let (x_name, y_name) = sim_result.plane().axes_names();
                    let (xs, ys, vals) = sim_result.into_components();
                    let mut min = 0_f32;
                    let mut max = 0_f32;
                    vals.iter().for_each(|row| {
                        row.iter().for_each(|&val| {
                            min = min.min(val);
                            max = max.max(val);
                        })
                    });
                    renderer.call1(
                        "render_2d",
                        (xs, x_name, ys, y_name, vals, "Wavefunction Value", min, max),
                    )?;
                    Ok(())
                },
            )?;
        }
    }

    Ok(())
}
