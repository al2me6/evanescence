from typing import List, Optional

import plotly.express as px
import plotly.graph_objects as go
from plotly.graph_objects import Figure


def normalize(
    source_min: float,
    source_max: float,
    dest_min: float,
    dest_max: float,
    val: float
) -> float:
    return (val - source_min) / (source_max - source_min) * (dest_max - dest_min) + dest_min


def render_pointillist_into_fig(
        fig: Figure,  # type: ignore
        xs: List[float],
        ys: List[float],
        zs: List[float],
        vals: List[float],
        min_size: float = 0.8,
        max_size: float = 1.5,
        colors: Optional[List[float]] = None,
        colorscale: Optional[str] = None,
) -> Figure:  # type: ignore
    abs_vals = tuple(abs(val) for val in vals)
    max_val = max(abs_vals)
    min_val = min(abs_vals)
    fig.add_trace(go.Scatter3d(  # type: ignore
        x=xs,
        y=ys,
        z=zs,
        mode="markers",
        marker=dict(
            size=tuple(
                normalize(min_val, max_val, min_size, max_size, val)
                for val in abs_vals
            ),
            line=dict(width=0),  # Remove border on marker.
            color=colors or vals,
            colorscale=colorscale or "RdBu_r",
            cmid=0,  # Fix "colorless" to 0. Thus, red is + and blue is -.
            # opacity=0.98,  # Improve visibility of overlapping features.
            showscale=True,
        )
    ))
    fig.update_layout(  # type: ignore
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
    return fig


def render_pointillist(
        xs: List[float],
        ys: List[float],
        zs: List[float],
        vals: List[float],
) -> None:
    fig = go.Figure()  # type: ignore
    fig = render_pointillist_into_fig(fig, xs, ys, zs, vals)
    fig.show()  # type: ignore


def render_pointillist_with_nodes(
        xs_pt: List[float],
        ys_pt: List[float],
        zs_pt: List[float],
        vals_pt: List[float],
        xs_iso: List[float],
        ys_iso: List[float],
        zs_iso: List[float],
        vals_iso: List[float],
) -> None:
    fig = go.Figure()  # type: ignore
    fig.add_trace(go.Isosurface(  # type: ignore
        x=xs_iso,
        y=ys_iso,
        z=zs_iso,
        value=vals_iso,
        caps=dict(x_show=False, y_show=False, z_show=False),
        flatshading=False,
        opacity=0.075,
        isomin=0,
        isomax=0,
        surface_count=1,
        colorscale="greens",
    ))
    fig = render_pointillist_into_fig(fig, xs_pt, ys_pt, zs_pt, vals_pt)
    fig.show()  # type: ignore


def render_pointillist_complex(
        xs: List[float],
        ys: List[float],
        zs: List[float],
        vals_moduli: List[float],
        vals_arguments: List[float],
) -> None:
    fig = go.Figure()  # type: ignore
    fig = render_pointillist_into_fig(
        fig, xs, ys, zs, vals_moduli,
        min_size=0.2, max_size=1.5,
        colors=vals_arguments, colorscale="phase"
    )
    fig.show()  # type: ignore


def render_1d(xs: List[float], xs_label: str, ys: List[float], ys_label: str) -> None:
    fig = px.line(x=xs, y=ys, labels=dict(x=xs_label, y=ys_label))
    fig.show()


def render_2d(
        row: List[float],
        row_axis_label: str,
        col: List[float],
        col_axis_label: str,
        vals: List[List[float]],
        val_axis_label,
        min_val: float,
        max_val: float,
) -> None:
    fig = go.Figure(data=go.Surface(  # type: ignore
        x=row,
        y=col,
        z=vals,
        colorscale="RdYlBu_r",
        cmid=0,
        contours=dict(
            x_highlight=False,
            y_highlight=False,
            z=dict(
                start=min_val,
                end=max_val,
                size=(max_val-min_val)/19,
                show=True,
                project_z=True,
                usecolormap=True,
            )
        ),
        lighting=dict(
            diffuse=0.2,
            specular=0.05,
            roughness=1,
        )
    ))
    fig.update_layout(
        template="plotly_white",
        hovermode=False,
        dragmode="orbit",
        margin=dict(l=0, r=0, b=0, t=0),
        scene=dict(
            xaxis_title=row_axis_label,
            yaxis_title=col_axis_label,
            zaxis_title=val_axis_label,
            xaxis_showspikes=False,
            yaxis_showspikes=False,
            zaxis_showspikes=False,
        )
    )
    fig.show()
