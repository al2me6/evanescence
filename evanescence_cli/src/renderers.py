from typing import List

import plotly.express as px
import plotly.graph_objects as go


def render_pointillist(xs: List[float], ys: List[float], zs: List[float], vals: List[float]) -> None:
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


def render_1d(xs: List[float], ys: List[float]) -> None:
    fig = px.line(x=xs, y=ys)
    fig.show()
