#!/usr/bin/env pypy3
from __future__ import annotations

import webbrowser
from contextlib import redirect_stdout
from enum import Enum
from io import StringIO
from itertools import chain
from math import cos, exp, pi, sin, sqrt
from random import random, uniform
from typing import Callable, Dict, Iterator, NamedTuple, Sequence, Tuple, TypeVar

import plotly.express as px
import plotly.graph_objects as go

SQRT_PI_INV_OVER_2 = 1 / sqrt(pi) / 2

EvaluationResult = Tuple["Point", float]
Orbital = Callable[["Point"], EvaluationResult]
T = TypeVar("T")


class Quality(Enum):  # These values seem to produce reasonable-ish results.
    MINIMUM = 5_000  # Recognizable, but that's about it.
    LOW = 10_000
    MEDIUM = 25_000
    HIGH = 50_000
    VERY_HIGH = 100_000
    EXTREME = 200_000  # Over 30 seconds on a 2700X (single-threaded).


def take(it: Iterator[T], n: int) -> Iterator[T]:
    try:
        for _ in range(n):
            yield next(it)
    except StopIteration:
        pass


class Point(NamedTuple):
    x: float
    y: float
    z: float

    @property
    def norm(self) -> float:
        return sqrt(self.x*self.x + self.y*self.y + self.z*self.z)

    @classmethod
    def random_point_in_ball(cls, rho_max: float) -> Point:
        sin_theta = uniform(-1, 1)
        cos_theta = sqrt(1 - sin_theta**2)
        phi = uniform(0, 2 * pi)
        rho = random()**(1/3) * rho_max
        return cls(
            x=rho * cos_theta * cos(phi),
            y=rho * cos_theta * sin(phi),
            z=rho * sin_theta
        )

    @classmethod
    def random_points_in_ball(cls, rho_max: float) -> Iterator[Point]:
        while True:
            yield cls.random_point_in_ball(rho_max)


def orbital_2s(point: Point) -> EvaluationResult:
    rho = 2 * point.norm / 2
    R = (
        1 / (2 * sqrt(2))
        * (2 - rho)
        * exp(-rho / 2)
    )
    Y = SQRT_PI_INV_OVER_2
    return point, R * Y


def orbital_2px(point: Point) -> EvaluationResult:
    r = point.norm
    rho = 2 * r / 2
    R = (
        1 / (2 * sqrt(6))
        * rho
        * exp(-rho / 2)
    )
    Y = (
        SQRT_PI_INV_OVER_2
        * 1 / sqrt(3)
        * point.x / r
    )
    return point, R * Y


def orbital_3px(point: Point) -> EvaluationResult:
    r = point.norm
    rho = 2 * r / 3
    R = (
        1 / (9 * sqrt(6))
        * rho * (4 - rho)
        * exp(-rho / 2)
    )
    Y = (
        SQRT_PI_INV_OVER_2
        * sqrt(3)
        * point.x / r
    )
    return point, R * Y


def orbital_4dx2y2(point: Point) -> EvaluationResult:
    r = point.norm
    rho = 2 * r / 4
    R = (
        1 / (96 * sqrt(5))
        * (6 - rho) * rho**2
        * exp(-rho / 2)
    )
    Y = (
        SQRT_PI_INV_OVER_2
        * sqrt(15) / 2
        * (point.x**2 - point.y**2) / r**2
    )
    return point, R * Y


def orbital_4dz2(point: Point) -> EvaluationResult:
    r = point.norm
    rho = 2 * r / 4
    R = (
        1 / (96 * sqrt(5))
        * (6 - rho) * rho * rho
        * exp(-rho / 2)
    )
    Y = (
        SQRT_PI_INV_OVER_2
        * sqrt(5) / 2
        * (2 * point.z*point.z - point.x*point.x - point.y*point.y) / (r*r)
    )
    return point, R * Y


def orbital_5f_general_x_x2_3y2(point: Point) -> EvaluationResult:
    r = point.norm
    rho = 2 * r / 5
    R = (
        1 / (300 * sqrt(70))
        * (8 - rho) * rho**3
        * exp(-rho / 2)
    )
    Y = (
        SQRT_PI_INV_OVER_2
        * sqrt(70) / 4
        * point.x * (point.x**2 - 3 * point.y**2) / r**3
    )
    return point, R * Y


def orbital_5f_cubic_y3(point: Point) -> EvaluationResult:
    r = point.norm
    rho = 2 * r / 5
    R = (
        1 / (300 * sqrt(70))
        * (8 - rho) * rho**3
        * exp(-rho / 2)
    )
    Y = (
        SQRT_PI_INV_OVER_2
        * sqrt(7) / 2
        * point.y * (5 * point.y**2 - 3 * r**2) / r**3
    )
    return point, R * Y


def orbital_5f_general_zx2y2(point: Point) -> EvaluationResult:
    r = point.norm
    rho = 2 * r / 5
    R = (
        1 / (300 * sqrt(70))
        * (8 - rho) * rho**3
        * exp(-rho / 2)
    )
    Y = (
        SQRT_PI_INV_OVER_2
        * sqrt(105) / 2
        * point.z * (point.x**2 - point.y**2) / r**3
    )
    return point, R * Y


def monte_carlo(orbital: Orbital, estimation_sample_count: int) -> Iterator[EvaluationResult]:
    estimation_sample_count = max(10_000, estimation_sample_count)
    # Questionable heuristic for estimating the size of the orbital.
    # obtained by inspection of orbital sizes.
    # Note that the 9th char is the principal quantum number (this is a HACK!).
    rho_max = 8 * int(int(orbital.__name__[8])**1.5)
    sampler = map(orbital, Point.random_points_in_ball(rho_max))
    estimation_samples = (
        # Force the origin to be sampled to ensure that s-orbitals are estimated accurately.
        # Otherwise, the sharp spike near the origin is difficult to sample.
        # Offset slightly to avoid division-by-zero.
        tuple(orbital(Point(0, 0, 1E-6))),
        *take(sampler, estimation_sample_count - 1)
    )
    max_val = max(abs(val) for _, val in estimation_samples)
    print(f"Approx. max value {max_val}")
    for pt, val in chain(estimation_samples, sampler):
        if abs(val) / max_val > random():
            yield pt, val / max_val


def collect_results(
        points: Iterator[EvaluationResult]
        # Python ought to have type annotations generic over values...
) -> Tuple[Sequence[float], Sequence[float], Sequence[float], Sequence[float]]:
    xs = []
    ys = []
    zs = []
    vals = []
    for pt, val in points:
        xs.append(pt.x)
        ys.append(pt.y)
        zs.append(pt.z)
        vals.append(val)
    return xs, ys, zs, vals


def simulate_and_plot(orbital: Orbital, quality: Quality):
    xs, ys, zs, vals = collect_results(take(
        monte_carlo(orbital, quality.value),
        quality.value
    ))

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
        dragmode="orbit",
    )
    output_filename = f"renders/{orbital.__name__}_{quality.name}.html"
    fig.write_html(output_filename, include_plotlyjs="cdn")
    # webbrowser.open(output_filename)


def plot_max_estimate_accuracies(num_runs: int) -> None:  # This is naaaaasty.
    for orbital in (
        obj for name, obj in globals().items()
        if name.startswith("orbital_")
    ):
        print(f"Generating plot for orbital function {orbital.__name__}")
        hist_data: Dict[str, Tuple[float, ...]] = {}
        for num_samples in range(2500, 22500, 2500):
            buf = StringIO()
            for _ in range(num_runs):
                with redirect_stdout(buf):
                    next(monte_carlo(orbital, estimation_sample_count=num_samples))
            # The most inelegant method for obtaining this value possible.
            estimated_max_vals = tuple(map(
                lambda s: float(s[18:].strip()),
                buf.getvalue().strip().split("\n")
            ))
            hist_data[f"{num_samples} samples"] = estimated_max_vals
        fig = px.histogram(
            hist_data,
            title=f"Estimated Max Wavefunction Value, {orbital.__name__}, {num_runs} Runs",
            x=list(hist_data.keys()),
            barmode="group",
            nbins=10,
            template="plotly_white"
        )
        fig.write_image(
            f"max_estimate_accuracy/estimated_max_distr_{orbital.__name__}.png",
            scale=2,
        )


if __name__ == "__main__":
    simulate_and_plot(orbital_4dx2y2, Quality.VERY_HIGH)
    # plot_max_estimate_accuracies(200)
