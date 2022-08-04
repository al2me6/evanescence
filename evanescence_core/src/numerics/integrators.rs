/// A potato trapezoidal integrator computing the integral of `y dx` given existing samples.
///
/// `xs` and `ys` must have the same length, and `xs` should be monotonically increasing.
pub fn integrate_trapezoidal(xs: &[f32], ys: &[f32]) -> f32 {
    itertools::zip_eq(xs.array_windows(), ys.array_windows())
        .map(|([x1, x2], [y1, y2])| (y1 + y2) * 0.5 * (x2 - x1))
        .sum()
}

/// Perform a single step of RK4 integration at step size `h`, accumulating into `t` and `y`.
///
/// # Panics
/// Panics if the step size `h` is not positive.
pub fn integrate_rk4_step(dfdt: impl Fn(f32) -> f32, t: &mut f32, y: &mut f32, h: f32) {
    const FRAC_1_6: f32 = 0.166_666_67;

    assert!(h > 0_f32);

    let k_1 = dfdt(*t);
    let k_2 = dfdt(*t + 0.5 * h);
    let k_4 = dfdt(*t + h);
    *y += FRAC_1_6 * h * (k_1 + 4_f32 * k_2 + k_4);
    *t += h;
}

/// Integrate `df(t)/dt` on `[t_0, t_f]` with step size `h` using the RK4 method.
///
/// # Panics
/// Panics if the step size `h` is not positive, or if the interval of integration is backwards.
pub fn integrate_rk4(dfdt: impl Fn(f32) -> f32, t_0: f32, t_f: f32, mut h: f32) -> f32 {
    assert!(t_0 < t_f);
    assert!(h > 0_f32);

    let mut t = t_0;
    let mut y = 0_f32;
    while t < t_f {
        h = h.min(t_f - t);
        integrate_rk4_step(&dfdt, &mut t, &mut y, h);
    }
    y
}

/// Evaluate a multiple integral as an iterated integral using [`integrate_rk4`]. The integrals are
/// evaluated in the order that the bounds for the variables are listed.
///
/// Syntax:
/// ```ignore
/// integrate_rk4_multiple!(
///     <expression in several variables>,
///     step: <step size>,
///     <first variable>: (<lower bound>, <upper bound>),
///     // etc.
/// )
/// ```
#[macro_export]
macro_rules! integrate_rk4_multiple {
    (
        $f:expr,
        step : $h:expr,
        $var:ident : ($t_0:expr , $t_f:expr)
        $(,)?
    ) => {
        $crate::numerics::integrators::integrate_rk4(|$var| $f, $t_0, $t_f, $h)
    };
    (
        $f:expr,
        step : $h:expr,
        $var:ident : ($t_0:expr , $t_f:expr) ,
        $($var_tail:ident : $range_tail:tt),+
        $(,)?
    ) => {
        $crate::numerics::integrators::integrate_rk4(
            |$var| integrate_rk4_multiple!(
                $f,
                step : $h,
                $($var_tail : $range_tail),+
            ),
            $t_0,
            $t_f,
            $h,
        )
    }
}

#[cfg(test)]
mod tests {
    use approx::{assert_abs_diff_eq, assert_relative_eq};

    use super::{integrate_rk4, integrate_trapezoidal};
    use crate::geometry::Linspace;

    #[test]
    fn trapezoidal() {
        let xs = (-2_f32..=4.0).linspace(100).collect::<Vec<_>>();

        assert_abs_diff_eq!(
            integrate_trapezoidal(
                &xs,
                &xs.iter()
                    .map(|x| x.powi(3) - 3.0 * x * x - 2.0 * x + 4.0)
                    .collect::<Vec<_>>()
            ),
            0.0,
            epsilon = 5E-5,
        );

        assert_relative_eq!(
            integrate_trapezoidal(
                &xs,
                &xs.iter()
                    .map(|x| (x - 2.0).exp() + x.cos())
                    .collect::<Vec<_>>()
            ),
            7.523_235,
            max_relative = 5E-4,
        );
    }

    #[test]
    fn rk4() {
        assert_relative_eq!(integrate_rk4(|x| x, 0.0, 5.0, 1.0), 12.5);

        assert_relative_eq!(
            integrate_rk4(
                |x| x.powi(3) / 3.0 - 2.0 * x + 0.5_f32.powf(x),
                -5.0,
                5.0,
                0.1,
            ),
            46.121_16,
            max_relative = 1E-6,
        );
    }

    #[test]
    fn rk4_triple() {
        assert_relative_eq!(
            integrate_rk4_multiple!(
                x * x * y + z.cos(),
                step: 0.1,
                x: (0., 1.),
                y: (-2., -1.),
                z: (-1., 1.),
            ),
            0.682_942,
        );
    }
}
