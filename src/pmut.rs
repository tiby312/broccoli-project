//! Provides a mutable pointer type that is more restrictive that `&mut T`, in order
//! to protect tree invariants.
//! [`PMut`] is short for protected mutable reference.
//!
//! ```rust
//! use broccoli::{prelude::*,bbox,rect};
//!
//!
//! let mut bots=[bbox(rect(0,10,0,10),0)];
//! let mut tree=broccoli::new(&mut bots);
//!
//! tree.find_colliding_pairs_mut(|a,b|{
//!    //We cannot allow the user to swap these two
//!    //bots. They should be allowed to mutate
//!    //whats inside each of them (aside from their aabb),
//!    //but not swap.
//!
//!    //core::mem::swap(a,b); // We cannot allow this!!!!
//!
//!    //This is allowed.
//!    core::mem::swap(a.unpack_inner(),b.unpack_inner());
//! })
//!
//! ```

use crate::inner_prelude::*;

///Trait exposes an api where you can return a read-only reference to the axis-aligned bounding box
///and at the same time return a mutable reference to a seperate inner section.
///
///The trait in unsafe since an incorrect implementation could allow the user to get mutable
///references to each element in the tree allowing them to swap them and thus violating
///invariants of the tree. This can be done if the user were to implement with type `Inner=Self`
///
///We have no easy way to ensure that the Inner type only points to the inner portion of a AABB
///so we mark this trait as unsafe.
pub unsafe trait HasInner: Aabb {
    type Inner;

    fn get_inner_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner);
}

///See the pmut module documentation for more explanation.
#[repr(transparent)]
pub(crate) struct PMutPtr<T: ?Sized> {
    _inner: *mut T,
}

unsafe impl<T: ?Sized> Send for PMutPtr<T> {}
unsafe impl<T: ?Sized> Sync for PMutPtr<T> {}

///A protected mutable reference. A protected mutable reference that derefs to `&T`.
///See the pmut module documentation for more explanation.
#[repr(transparent)]
pub struct PMut<'a, T: ?Sized> {
    inner: &'a mut T,
}

impl<'a, T: ?Sized> core::ops::Deref for PMut<'a, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        self.inner
    }
}





impl<'a,'b:'a,T> PMut<'a,PMut<'b,T>>{
    #[inline(always)]
    pub fn into_inner(self)->PMut<'a,T>{
        PMut::new(self.inner.inner)
    }
}
impl<'a,T> PMut<'a,T>{
    #[inline(always)]
    pub fn into_slice(self)->PMut<'a,[T]>{
        PMut{inner:core::slice::from_mut(self.inner)}   
    }
}
impl<'a, T: ?Sized> PMut<'a, T> {
    ///Create a PMut
    #[inline(always)]
    pub fn new(inner: &'a mut T) -> PMut<'a, T> {
        PMut { inner }
    }

    ///Start a new borrow lifetime
    #[inline(always)]
    pub fn borrow_mut(&mut self) -> PMut<T> {
        PMut { inner: self.inner }
    }

    ///Manually convert to read only reference.
    #[inline(always)]
    pub fn into_ref(self) -> &'a T {
        self.inner
    }
}


impl<'a,'b:'a,T:Aabb> PMut<'a,NodeMut<'b,T>>{
    #[inline(always)]
    pub fn into_range(self) -> PMut<'a,[T]> {
        self.inner.range.borrow_mut()
    }
}

impl<'a, T: HasInner> PMut<'a, T> {
    ///Unpack for the read-only rect and the mutable inner component
    #[inline(always)]
    pub fn unpack(self) -> (&'a Rect<T::Num>, &'a mut T::Inner) {
        self.inner.get_inner_mut()
    }
    ///Unpack only the mutable innner component
    #[inline(always)]
    pub fn unpack_inner(self) -> &'a mut T::Inner {
        self.inner.get_inner_mut().1
    }

    ///Get a mutable reference to only the innner component
    #[inline(always)]
    pub fn inner_mut(&mut self) -> &mut T::Inner {
        self.inner.get_inner_mut().1
    }
}

unsafe impl<'a, T: Aabb> Aabb for PMut<'a, T> {
    type Num = T::Num;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        self.inner.get()
    }
}

impl<'a, T> PMut<'a, [T]> {

    ///We can't use the index trait because we don't want
    ///to return a mutable reference.
    #[inline(always)]
    pub fn get_index_mut(self, ind: usize) -> PMut<'a,T> {
        PMut::new(&mut self.inner[ind])
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
