from typing import List

import plotly.express as px
import plotly.graph_objects as go


def render_pointillist(
        xs: List[float],
        ys: List[float],
        zs: List[float],
        vals: List[float],
) -> None:
    fig = go.Figure(data=go.Scatter3d(  # type: ignore
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
            showscale=True,
        ),
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


def render_1d(xs: List[float], xs_label: str, ys: List[float], ys_label: str) -> None:
    fig = px.line(x=xs, y=ys, labels=dict(x=xs_label, y=ys_label))
    fig.show()


def render_2d(
        row: List[float],
        row_axis_label: str,
        col: List[float],
        col_axis_label: str,
        vals: List[float],
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
