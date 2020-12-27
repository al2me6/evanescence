use std::time::Instant;

use anyhow::{Context, Result};
use argh::FromArgs;
use evanescence_core::{
    monte_carlo::{MonteCarlo, Quality},
    orbital::{QuantumNumbers, RealOrbital},
};
use indoc::indoc;
use pyo3::{prelude::*, types::PyModule};

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
    #[argh(option, short = 'q', default = "Quality::High")]
    /// render quality: Minimum, Low, Medium, High (default), VeryHigh, or Extreme
    quality: Quality,
    #[argh(switch)]
    /// skip rendering (effectively a benchmark for computation speed)
    skip_render: bool,
}

fn main() -> Result<()> {
    let Args { n, l, m, quality, skip_render } = argh::from_env();

    let qn = QuantumNumbers::new(n, l, m).with_context(|| {
        format!(
            "received illegal quantum numbers: n={}, l={}, m={}; must satisfy n > l and l >= |m|",
            n, l, m
        )
    })?;
    println!("Rendering real orbital {} at {} quality...", qn, quality);

    let now = Instant::now();
    let sim_result = RealOrbital::monte_carlo_simulate(qn, quality);
    println!(
        "Simulated {} points in {:.3}s.",
        quality as u32,
        now.elapsed().as_secs_f64()
    );

    if skip_render {
        println!("Skipping rendering.");
        return Ok(())
    }

    let now = Instant::now();
    // We are stuck with Python interop until plotly.rs implements support for 3D scatterplots.
    // See https://github.com/igiagkiozis/plotly/pull/30.
    Python::with_gil(|py| -> PyResult<()> {
        let renderer = PyModule::from_code(
            py,
            indoc! {r##"
                from typing import List
                import plotly.graph_objects as go

                def render(xs: List[float], ys: List[float], zs: List[float], vals: List[float]):
                    fig = go.Figure(data=go.Scatter3d(
                        x=xs,
                        y=ys,
                        z=zs,
                        mode="markers",
                        marker=dict(
                            size=tuple(min(abs(val) / 2 + 1, 2) for val in vals),
                            line=dict(width=0),  # Remove border on marker.
                            color=vals,
                            colorscale="RdBu_r",
                            cmid=0,  # Fix "colorless" to 0. Thus, red is + and blue is -.
                            opacity=0.98,  # Improve visibility of overlapping features.
                        ),
                        hovertext=vals,
                    ))
                    fig.update_layout(
                        template="plotly_white",
                        hovermode=False,
                        dragmode="orbit",
                        margin=dict(l=0, r=0, b=0, t=0),
                        scene=dict(
                            xaxis_showspikes=False,
                            yaxis_showspikes=False,
                            zaxis_showspikes=False,
                        )
                    )
                    fig.show()
            "##},
            "renderer.py",
            "renderer",
        )?;
        renderer.call1("render", sim_result.into_components())?;
        Ok(())
    })?;
    println!("Rendered in {:.3}s.", now.elapsed().as_secs_f64());

    Ok(())
}
