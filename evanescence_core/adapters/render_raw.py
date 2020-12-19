#!/usr/bin/python3
"""Raw point cloud data renderer.

Takes points of the form `(x, y, z), val`, separated by newlines, from stdin.
"""

from sys import stdin

import plotly.graph_objects as go

xs = []
ys = []
zs = []
vals = []

for line in stdin.readlines():
    for arr, val in zip((xs, ys, zs, vals),
                        line.replace("(", "").replace(")", "").strip().split(",")):
        arr.append(float(val.strip()))

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
    # hovermode=False,
    dragmode="orbit",
    margin=dict(l=0, r=0, b=0, t=0),
    scene=dict(
        xaxis_showspikes=False,
        yaxis_showspikes=False,
        zaxis_showspikes=False,
    )
)
fig.show()
