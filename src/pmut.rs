//! Provides a mutable pointer type that is more restrictive that &mut T, in order
//! to protect tree invariants.
//! PMut is short for protected mutable reference.
//!
//! It prevents the user from violating the invariants of the tree.
//!
//! ```rust
//! extern crate dinotree_alg;
//! extern crate axgeom;
//! use dinotree_alg::*;
//!
//!
//! let mut bots=[BBox::new(axgeom::Rect::new(0,10,0,10),0)];
//! let mut tree=DinoTree::new(&mut bots);
//!
//! tree.find_intersections_pmut(|mut a,mut b|{
//!    //We cannot allow the user to swap these two
//!    //bots. They should be allowed to mutate
//!    //whats inside each of them (aside from their aabb),
//!    //but not swap.
//!
//!    //core::mem::swap(a,b); // We cannot allow this!!!!
//!
//!    //This is allowed.
//!    core::mem::swap(a.inner_mut(),b.inner_mut());
//! })
//!
//! ```

use crate::inner_prelude::*;

///A protected mutable reference.
///See the pmut module documentation for more explanation.
#[repr(transparent)]
pub(crate) struct PMutPtr<T: ?Sized> {
    pub(crate) inner: core::ptr::NonNull<T>, //TODO make this private
}

unsafe impl<T: ?Sized> Send for PMutPtr<T> {}
unsafe impl<T: ?Sized> Sync for PMutPtr<T> {}


///A protected mutable reference.
///See the pmut module documentation for more explanation.
#[repr(transparent)]
pub struct PMut<'a, T: ?Sized> {
    pub(crate) inner: &'a mut T, //TODO make this private
}

impl<'a, T: ?Sized> PMut<'a, T> {
    #[inline(always)]
    pub(crate) fn as_ptr(&mut self) -> PMutPtr<T> {
        PMutPtr {
            inner: unsafe { core::ptr::NonNull::new_unchecked(self.inner as *mut _) },
        }
    }

    #[inline(always)]
    pub fn new(inner: &'a mut T) -> PMut<'a, T> {
        PMut { inner }
    }
    #[inline(always)]
    pub fn as_mut(&mut self) -> PMut<T> {
        PMut { inner: self.inner }
    }

    #[inline(always)]
    pub fn as_ref(&self) -> &T {
        self.inner
    }
}

impl<'a, T: Node> PMut<'a, T> {
    #[inline(always)]
    pub fn get(self) -> NodeRef<'a, T::T> {
        self.inner.get()
    }

    #[inline(always)]
    pub fn get_mut(self) -> NodeRefMut<'a, T::T> {
        self.inner.get_mut()
    }
}

impl<'a, T: HasInner> PMut<'a, T> {
    #[inline(always)]
    pub fn unpack(self) -> (&'a Rect<T::Num>, &'a mut T::Inner) {
        self.inner.get_inner_mut()
    }
}

unsafe impl<'a, T: Aabb> Aabb for PMut<'a, T> {
    type Num = T::Num;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        self.inner.get()
    }
}

unsafe impl<'a, T: HasInner> HasInner for PMut<'a, T> {
    type Inner = T::Inner;
    #[inline(always)]
    fn get_inner(&self) -> (&Rect<T::Num>, &Self::Inner) {
        self.inner.get_inner()
    }

    #[inline(always)]
    fn get_inner_mut(&mut self) -> (&Rect<T::Num>, &mut Self::Inner) {
        self.inner.get_inner_mut()
    }
}

impl<'a, T: ?Sized> core::ops::Deref for PMut<'a, T> {
    type Target = &'a T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self as *const _ as *const _) }
    }
}

//TODO use this
impl<'a, T: HasInner> PMut<'a, T> {
    #[inline(always)]
    pub fn into_inner(self) -> &'a mut T::Inner {
        self.inner.get_inner_mut().1
    }
}
impl<'a, T> PMut<'a, [T]> {
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[inline(always)]
    pub fn split_first_mut(self) -> Option<(PMut<'a, T>, PMut<'a, [T]>)> {
        self.inner
            .split_first_mut()
            .map(|(first, inner)| (PMut { inner: first }, PMut { inner }))
    }

    #[inline(always)]
    pub fn truncate_to(self, a: core::ops::RangeTo<usize>) -> Self {
        PMut {
            inner: &mut self.inner[a],
        }
    }
    #[inline(always)]
    pub fn truncate_from(self, a: core::ops::RangeFrom<usize>) -> Self {
        PMut {
            inner: &mut self.inner[a],
        }
    }

    #[inline(always)]
    pub fn truncate(self, a: core::ops::Range<usize>) -> Self {
        PMut {
            inner: &mut self.inner[a],
        }
    }

    #[inline(always)]
    pub fn iter(self) -> core::slice::Iter<'a, T> {
        self.inner.iter()
    }
    #[inline(always)]
    pub fn iter_mut(self) -> PMutIter<'a, T> {
        PMutIter {
            inner: self.inner.iter_mut(),
        }
    }
}

impl<'a, T> core::iter::IntoIterator for PMut<'a, [T]> {
    type Item = PMut<'a, T>;
    type IntoIter = PMutIter<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

///Iterator produced by `PMut<[T]>` that generates `PMut<T>`
pub struct PMutIter<'a, T> {
    inner: core::slice::IterMut<'a, T>,
}
impl<'a, T> Iterator for PMutIter<'a, T> {
    type Item = PMut<'a, T>;

    #[inline(always)]
    fn next(&mut self) -> Option<PMut<'a, T>> {
        self.inner.next().map(|inner| PMut { inner })
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, T> core::iter::FusedIterator for PMutIter<'a, T> {}
impl<'a, T> core::iter::ExactSizeIterator for PMutIter<'a, T> {}

impl<'a, T> DoubleEndedIterator for PMutIter<'a, T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|inner| PMut { inner })
    }
}
