use broccoli::tree::build::*;

///
///
/// # Safety
///
/// Aabb::get() guarenteed to return the same value while pinned by `TreePin`.
///
///
pub unsafe trait TrustedAabb: Aabb {}

unsafe impl<N: Num, T> TrustedAabb for BBox<N, T> {}
unsafe impl<N: Num, T> TrustedAabb for BBoxMut<'_, N, T> {}
unsafe impl<N: Num> TrustedAabb for Rect<N> {}
unsafe impl<T: TrustedAabb> TrustedAabb for &T {}
unsafe impl<T: TrustedAabb> TrustedAabb for &mut T {}
