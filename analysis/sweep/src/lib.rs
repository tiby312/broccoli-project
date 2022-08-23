use broccoli::{aabb::pin::AabbPin, aabb::Aabb, build::default_axis};

impl<'a, T: Aabb> SweepAndPrune<'a, T> {
    pub fn find_colliding_pairs(&mut self, mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        let mut prevec = Vec::with_capacity(2048);
        let bots = AabbPin::from_mut(self.inner);
        broccoli::queries::colfind::oned::find_2d(
            &mut prevec,
            default_axis(),
            bots,
            &mut func,
            true,
        );
    }
}

///
/// Sweep and prune collision finding algorithm
///
pub struct SweepAndPrune<'a, T> {
    inner: &'a mut [T],
}

impl<'a, T: Aabb> SweepAndPrune<'a, T> {
    pub fn new(inner: &'a mut [T]) -> Self {
        let axis = default_axis();
        broccoli::build::sweeper_update(axis, inner);
        SweepAndPrune { inner }
    }
}
