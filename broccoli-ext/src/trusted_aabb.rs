use broccoli::tree::node::*;

///Indicates that the user supplied a rectangle
///that intersects with a another one previously queries
///in the session.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct RectIntersectErr;

///
///
/// # Safety
///
/// Aabb::get() guarenteed to return the same value while pinned by `TreePin`.
///
///
pub unsafe trait TrustedAabb: Aabb {}

unsafe impl<N: Num, T> TrustedAabb for BBox<N, T> {}
unsafe impl<N: Num> TrustedAabb for Rect<N> {}
unsafe impl<T: TrustedAabb> TrustedAabb for &T {}
unsafe impl<T: TrustedAabb> TrustedAabb for &mut T {}
