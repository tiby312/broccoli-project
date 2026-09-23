//!
//! Contains AABB primitives and building blocks.
//!

pub mod pin;

use super::*;
use pin::*;

pub use axgeom::Range;
pub use axgeom::Rect;

///
/// Singifies this object can be swapped around in a slice
/// many times without much of a performance hit.
///
pub trait ManySwap {}

impl<N> ManySwap for Rect<N> {}
impl<'a, N> ManySwap for &'a mut Rect<N> {}

impl<'a, N, T> ManySwap for &'a mut (Rect<N>, T) {}

impl<'a, N, T> ManySwap for (Rect<N>, &'a mut T) {}
impl<'a, N, T> ManySwap for (Rect<N>, &'a T) {}

impl<N> ManySwap for (Rect<N>, ()) {}
impl<N> ManySwap for (Rect<N>, usize) {}
impl<N> ManySwap for (Rect<N>, u32) {}
impl<N> ManySwap for (Rect<N>, u64) {}

impl<'a, N, T> ManySwap for BBoxDirect<N, &'a mut T> {}
impl<N> ManySwap for BBoxDirect<N, ()> {}
impl<N> ManySwap for BBoxDirect<N, u32> {}
impl<N> ManySwap for BBoxDirect<N, u64> {}
impl<N> ManySwap for BBoxDirect<N, usize> {}

///
/// Wrapper to opt in to being allowed to be fed to swap intensive algorithms.
///
#[derive(Copy, Clone, Debug)]
pub struct ManySwappable<T>(pub T);
impl<T> ManySwap for ManySwappable<T> {}

impl<T> ManySwap for &mut ManySwappable<T> {}

// impl<T: Aabb> Aabb for &mut ManySwappable<T> {
//     type Num = T::Num;
//     #[inline(always)]
//     fn get(&self) -> &Rect<Self::Num> {
//         self.0.get()
//     }
// }

// impl<T:HasInner> HasInner for &mut ManySwappable<T>{
//     type Inner<'a> = T::Inner<'a> where T:'a;
//     fn inner<'a>(&'a mut self)->Self::Inner<'a> {
//         self.0.inner()
//     }
// }

impl<T: HasInner> HasInner for ManySwappable<T> {
    type Inner<'a>
        = T::Inner<'a>
    where
        T: 'a;
    fn inner<'a>(&'a mut self) -> Self::Inner<'a> {
        self.0.inner()
    }
}

impl<T: Aabb> Aabb for ManySwappable<T> {
    type Num = T::Num;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        self.0.get()
    }
}

/// The underlying number type used for the tree.
/// It is auto implemented by all types that satisfy the type constraints.
/// Notice that no arithmetic is possible. The tree is constructed
/// using only comparisons and copying.
pub trait Num: PartialOrd + Copy + Default + std::fmt::Debug {}
impl<T> Num for T where T: PartialOrd + Copy + Default + std::fmt::Debug {}



impl<N: Num, T> HasInner for (Rect<N>, T) {
    type Inner<'a>
        = &'a mut T
    where
        Self: 'a;
    fn inner<'a>(&'a mut self) -> Self::Inner<'a> {
        &mut self.1
    }
}
impl<N: Num, T> Aabb for (Rect<N>, T) {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        &self.0
    }
}

impl<T:HasInner> HasInner for &mut T{
    type Inner<'a>
        = T::Inner<'a>
    where
        Self: 'a;
    fn inner<'a>(&'a mut self) -> Self::Inner<'a> {
        (**self).inner()
    }
}


impl<T:Aabb> Aabb for &mut T{
    type Num= T::Num;

    fn get(&self) -> &Rect<Self::Num> {
        (**self).get()
    }
}

///
/// Trait to signify that this object has an axis aligned bounding box.
///
pub trait Aabb {
    type Num: Num;

    fn get(&self) -> &Rect<Self::Num>;
}

pub trait HasInner {
    type Inner<'a>
    where
        Self: 'a;
    fn inner<'a>(&'a mut self) -> Self::Inner<'a>;
}

pub(crate) trait AabbExt: Aabb {
    #[inline(always)]
    fn range<A: Axis>(&self, a: A) -> &axgeom::Range<Self::Num> {
        self.get().get_range(a)
    }
}
impl<T: Aabb> AabbExt for T {}

impl<N:Num> HasInner for Rect<N>{
    type Inner<'a>=() where N: 'a;

    fn inner<'a>(&'a mut self)->Self::Inner<'a>{
        ()
    }
}
impl<N: Num> Aabb for Rect<N> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        self
    }
}

/// A bounding box container object that implements [`Aabb`] and [`HasInner`].
/// Note that `&mut BBox<N,T>` also implements [`Aabb`] and [`HasInner`].
///
/// Using this one struct the user can construct the following types for bboxes to be inserted into the tree:
///
///* `BBox<N,T>`  (direct)
///* `&mut BBox<N,T>` (indirect)
///* `BBox<N,&mut T>` (rect direct, T indirect)
#[derive(Debug, Copy, Clone)]
pub struct BBoxDirect<N, T> {
    pub rect: Rect<N>,
    pub inner: T,
}

impl<N, T> BBoxDirect<N, T> {
    /// Constructor. Also consider using [`crate::bbox()`]
    #[inline(always)]
    #[must_use]
    pub fn new(rect: Rect<N>, inner: T) -> BBoxDirect<N, T> {
        BBoxDirect { rect, inner }
    }

    pub fn many_swap(self) -> ManySwappable<Self> {
        ManySwappable(self)
    }
}

impl<N: Num, T> HasInner for BBoxDirect<N, T> {
    type Inner<'a>
        = &'a mut T
    where
        Self: 'a;
    fn inner<'a>(&'a mut self) -> Self::Inner<'a> {
        &mut self.inner
    }
}

impl<N: Num, T> Aabb for BBoxDirect<N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        &self.rect
    }
}

///
/// BBox with a reference.
///
/// Similar to `BBox<N,&mut T>` except
/// get_inner_mut() doesnt return a `&mut &mut T`
/// but instead just a `&mut T`.
///
pub struct BBoxMut<'a, N, T> {
    pub rect: axgeom::Rect<N>,
    pub inner: &'a mut T,
}
impl<'a, N, T> ManySwap for BBoxMut<'a, N, T> {}

impl<'a, N, T> BBoxMut<'a, N, T> {
    /// Constructor. Also consider using [`crate::bbox()`]
    #[inline(always)]
    #[must_use]
    pub fn new(rect: Rect<N>, inner: &'a mut T) -> BBoxMut<'a, N, T> {
        BBoxMut { rect, inner }
    }
}
impl<N: Num, T> HasInner for BBoxMut<'_, N, T> {
    type Inner<'a>
        = &'a mut T
    where
        Self: 'a;
    fn inner<'a>(&'a mut self) -> Self::Inner<'a> {
        &mut self.inner
    }
}
impl<N: Num, T> Aabb for BBoxMut<'_, N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &axgeom::Rect<N> {
        &self.rect
    }
}
