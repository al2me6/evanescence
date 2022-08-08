/// A potato trapezoidal integrator computing the integral of `y dx` given existing samples.
///
/// `xs` and `ys` must have the same length, and `xs` should be monotonically increasing.
pub fn integrate_trapezoidal(xs: &[f32], ys: &[f32]) -> f32 {
    itertools::zip_eq(xs.array_windows(), ys.array_windows())
        .map(|([x1, x2], [y1, y2])| (y1 + y2) * 0.5 * (x2 - x1))
        .sum()
}

/// Perform a single step of Simpson's method at step size `h`, accumulating into `x` and `y`.
///
/// # Panics
/// Panics if the step size `h` is not positive.
pub fn integrate_simpson_step(mut dfdx: impl FnMut(f32) -> f32, x: &mut f32, y: &mut f32, h: f32) {
    const FRAC_1_6: f32 = 0.166_666_67;

    assert!(h > 0_f32);

    let k_1 = dfdx(*x);
    let k_2 = dfdx(*x + 0.5 * h);
    let k_4 = dfdx(*x + h);
    *y += FRAC_1_6 * h * (k_1 + 4_f32 * k_2 + k_4);
    *x += h;
}

/// Integrate `df(x)/dx` on `[x_0, x_f]` with step size `h` using Simpson's method.
///
/// # Panics
/// Panics if the step size `h` is not positive, or if the interval of integration is backwards.
pub fn integrate_simpson(mut dfdx: impl FnMut(f32) -> f32, x_0: f32, x_f: f32, mut h: f32) -> f32 {
    assert!(x_0 <= x_f);
    assert!(h > 0_f32);

    #[allow(clippy::float_cmp)]
    if x_0 == x_f {
        return 0.;
    }

    let mut x = x_0;
    let mut y = 0_f32;
    while x < x_f {
        h = h.min(x_f - x);
        integrate_simpson_step(&mut dfdx, &mut x, &mut y, h);
    }
    y
}

/// Evaluate a multiple integral as an iterated integral using [`integrate_simpson`]. The integrals
/// are evaluated in the order that the bounds for the variables are listed.
///
/// Syntax:
/// ```ignore
/// integrate_simpson_multiple!(
///     <expression in several variables>,
///     step: <step size>,
///     <first variable>: (<lower bound>, <upper bound>),
///     // etc.
/// )
/// ```
#[macro_export]
macro_rules! integrate_simpson_multiple {
    (
        $f:expr,
        step : $h:expr,
        $var:ident : ($x_0:expr , $x_f:expr)
        $(,)?
    ) => {
        $crate::numerics::integrators::integrate_simpson(|$var| $f, $x_0, $x_f, $h)
    };
    (
        $f:expr,
        step : $h:expr,
        $var:ident : ($x_0:expr , $x_f:expr) ,
        $($var_tail:ident : $range_tail:tt),+
        $(,)?
    ) => {
        $crate::numerics::integrators::integrate_simpson(
            |$var| integrate_simpson_multiple!(
                $f,
                step : $h,
                $($var_tail : $range_tail),+
            ),
            $x_0,
            $x_f,
            $h,
        )
    }
}

#[cfg(test)]
mod tests {
    use approx::{assert_abs_diff_eq, assert_relative_eq};

    use super::{integrate_simpson, integrate_trapezoidal};
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
    fn simpson() {
        assert_relative_eq!(integrate_simpson(|x| x, 0.0, 5.0, 1.0), 12.5);

        assert_relative_eq!(
            integrate_simpson(
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
    fn simpson_triple() {
        assert_relative_eq!(
            integrate_simpson_multiple!(
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
