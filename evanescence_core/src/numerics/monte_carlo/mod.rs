use crate::geometry::PointValue;

pub mod accept_reject;

pub trait MonteCarlo {
    type Output: Copy;

    fn simulate(&mut self, count: usize) -> Vec<PointValue<Self::Output>>;
}

/// Assert dyn-safety.
const _: Option<Box<dyn MonteCarlo<Output = f32>>> = None;
