use crate::inner_prelude::*;

///Shorthand constructor of [`BBox`]
#[inline(always)]
#[must_use]
pub fn bbox<N, T>(rect: Rect<N>, inner: T) -> BBox<N, T> {
    BBox::new(rect, inner)
}

///A bounding box container object that implements [`Aabb`] and [`HasInner`].
///Note that `&mut BBox<N,T>` also implements [`Aabb`] and [`HasInner`].
///
///Using this one struct the user can construct the following types for bboxes to be inserted into the tree:
///
///* `BBox<N,T>`  (direct)
///* `&mut BBox<N,T>` (indirect)
///* `BBox<N,&mut T>` (rect direct, T indirect)
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct BBox<N, T> {
    pub rect: Rect<N>,
    pub inner: T,
}

impl<N, T> BBox<N, T> {
    ///Constructor. Also consider using [`bbox()`]
    #[inline(always)]
    #[must_use]
    pub fn new(rect: Rect<N>, inner: T) -> BBox<N, T> {
        BBox { rect, inner }
    }
}

use core::convert::TryFrom;
impl<N: Copy, T> BBox<N, T> {
    ///Creates a `(Rect<N>,&mut T)` from a `(Rect<N>,T)`
    #[inline(always)]
    #[must_use]
    pub fn into_semi_direct(&mut self) -> BBox<N, &mut T> {
        BBox {
            rect: self.rect.clone(),
            inner: &mut self.inner,
        }
    }

    ///Simply returns a mutable reference
    #[inline(always)]
    #[must_use]
    pub fn into_indirect(&mut self) -> &mut BBox<N, T> {
        self
    }

    ///Change the number type of the Rect using
    ///promitive cast.
    #[inline(always)]
    #[must_use]
    pub fn inner_as<B: 'static + Copy>(self) -> BBox<B, T>
    where
        N: num_traits::AsPrimitive<B>,
    {
        BBox {
            rect: self.rect.inner_as(),
            inner: self.inner,
        }
    }

    ///Change the number type using `From`
    #[inline(always)]
    #[must_use]
    pub fn inner_into<A: From<N>>(self) -> BBox<A, T> {
        BBox {
            rect: self.rect.inner_into(),
            inner: self.inner,
        }
    }

    ///Change the number type using `TryFrom`
    #[inline(always)]
    #[must_use]
    pub fn inner_try_into<A: TryFrom<N>>(self) -> Result<BBox<A, T>, A::Error> {
        Ok(BBox {
            rect: self.rect.inner_try_into()?,
            inner: self.inner,
        })
    }
}

unsafe impl<N: Num, T> Aabb for BBox<N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        &self.rect
    }
}

unsafe impl<N: Num, T> HasInner for BBox<N, T> {
    type Inner = T;

    #[inline(always)]
    fn get_inner_mut(&mut self) -> (&Rect<N>, &mut Self::Inner) {
        (&self.rect, &mut self.inner)
    }
}

unsafe impl<N: Num, T> Aabb for &mut BBox<N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        &self.rect
    }
}

unsafe impl<N: Num, T> HasInner for &mut BBox<N, T> {
    type Inner = T;

    #[inline(always)]
    fn get_inner_mut(&mut self) -> (&Rect<N>, &mut Self::Inner) {
        (&self.rect, &mut self.inner)
    }
}
