//! Provides a mutable pointer type that is more restrictive that `&mut T`, in order
//! to protect tree invariants.
//! [`PMut`] is short for protected mutable reference.
//!
//! ```rust
//! use broccoli::{bbox,rect};
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

use crate::*;
use core::marker::PhantomData;

/// Trait exposes an api where you can return a read-only reference to the axis-aligned bounding box
/// and at the same time return a mutable reference to a separate inner section.
pub trait HasInner: Aabb {
    type Inner;
    fn get_inner_mut(&mut self) -> &mut Self::Inner;
}

/// A protected mutable reference that derefs to `&T`.
/// See the pmut module documentation for more explanation.
#[repr(transparent)]
#[derive(Debug)]
pub struct PMut<'a, T: ?Sized> {
    inner: *mut T,
    _p: PhantomData<&'a mut T>,
}
unsafe impl<T: ?Sized> Send for PMut<'_, T> {}
unsafe impl<T: ?Sized> Sync for PMut<'_, T> {}

impl<'a, T: ?Sized> core::ops::Deref for PMut<'a, T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { &*self.inner }
    }
}

impl<'a, T: ?Sized> From<&'a mut T> for PMut<'a, T> {
    fn from(a: &'a mut T) -> Self {
        PMut::new(a)
    }
}


impl<'a, T> PMut<'a, T> {
    /// Convert a `PMut<T>` inside a `PMut<[T]>` of size one.
    #[inline(always)]
    pub fn into_slice(self) -> PMut<'a, [T]> {
        PMut {
            inner: unsafe { core::slice::from_raw_parts_mut(self.inner, 1) as *mut _ },
            _p: PhantomData,
        }
    }
}
impl<'a, T: ?Sized> PMut<'a, T> {
    /// Create a protected pointer.
    #[inline(always)]
    pub fn new(inner: &'a mut T) -> PMut<'a, T> {
        PMut {
            inner: inner as *mut _,
            _p: PhantomData,
        }
    }

    /// Start a new borrow lifetime
    #[inline(always)]
    pub fn borrow_mut(&mut self) -> PMut<T> {
        PMut {
            inner: self.inner as *mut _,
            _p: PhantomData,
        }
    }

    #[inline(always)]
    pub fn into_ref(self) -> &'a T {
        unsafe { &*self.inner }
    }

    /// If this function were safe, it would
    /// defeat the purpose of this type.
    ///
    /// # Safety
    ///
    /// This is unsafe, since the user may mutate the inner AABB
    /// while T is inserted in a tree thus undoing the whole
    /// point of this struct.
    #[inline(always)]
    pub unsafe fn into_inner(self) -> &'a mut T {
        &mut *self.inner
    }
}

/// A destructured [`Node`]
pub struct NodeRef<'a, T: Aabb> {
    pub div: &'a Option<T::Num>,
    pub cont: &'a Range<T::Num>,
    pub range: PMut<'a, [T]>,
}

impl<'a, 'b: 'a, T: Aabb> PMut<'a, Node<'b, T>> {
    /// Destructure a node into its three parts.
    #[inline(always)]
    pub fn into_node_ref(self) -> NodeRef<'a, T> {
        unsafe {
            NodeRef {
                div: &(*self.inner).div,
                cont: &(*self.inner).cont,
                range: (*self.inner).range.borrow_mut(),
            }
        }
    }

    #[inline(always)]
    pub fn get_cont(&self) -> &Range<T::Num> {
        unsafe { &*(&self.cont as *const _) }
    }

    /// Return a mutable list of elements in this node.
    #[inline(always)]
    pub fn into_range(self) -> PMut<'a, [T]> {
        unsafe { (&mut *self.inner).range.borrow_mut() }
    }
}

impl<'a, T: Aabb> PMut<'a, T> {
    ///
    /// Return a read-only reference to the aabb.
    /// Note the lifetime.
    /// Given the guarantees of the [`Aabb`] trait, we can have this extended lifetime.
    ///
    pub fn rect(&self) -> &'a Rect<T::Num> {
        unsafe { &*(self.inner as *const T) }.get()
    }
    #[inline(always)]
    pub fn unpack_rect(self) -> &'a Rect<T::Num> {
        unsafe { { &(*self.inner) }.get() }
    }
}

impl<'a, T: HasInner> PMut<'a, T> {
    /// Unpack only the mutable innner component
    #[inline(always)]
    pub fn unpack_inner(self) -> &'a mut T::Inner {
        unsafe { (&mut *self.inner).get_inner_mut() }
    }
}

unsafe impl<'a, T: Aabb> Aabb for PMut<'a, T> {
    type Num = T::Num;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        unsafe { &*self.inner }.get()
    }
}

impl<'a, T> PMut<'a, [T]> {
    /// Return the element at the specified index.
    /// We can't use the index trait because we don't want
    /// to return a mutable reference.
    #[inline(always)]
    pub fn get_index_mut(self, ind: usize) -> PMut<'a, T> {
        PMut::new(unsafe { &mut (*self.inner)[ind] })
    }

    /// Split off the first element.
    #[inline(always)]
    pub fn split_first_mut(self) -> Option<(PMut<'a, T>, PMut<'a, [T]>)> {
        unsafe {
            self.into_inner().split_first_mut().map(|(first, inner)| {
                (
                    PMut {
                        inner: first as *mut _,
                        _p: PhantomData,
                    },
                    PMut {
                        inner: inner as *mut _,
                        _p: PhantomData,
                    },
                )
            })
        }
    }

    /// Return a smaller slice that ends with the specified index.
    #[inline(always)]
    pub fn truncate_to(self, a: core::ops::RangeTo<usize>) -> Self {
        unsafe {
            PMut {
                inner: &mut (*self.inner)[a],
                _p: PhantomData,
            }
        }
    }

    /// Return a smaller slice that starts at the specified index.
    #[inline(always)]
    pub fn truncate_from(self, a: core::ops::RangeFrom<usize>) -> Self {
        unsafe {
            PMut {
                inner: &mut (*self.inner)[a],
                _p: PhantomData,
            }
        }
    }

    /// Return a smaller slice that starts and ends with the specified range.
    #[inline(always)]
    pub fn truncate(self, a: core::ops::Range<usize>) -> Self {
        unsafe {
            PMut {
                inner: &mut (*self.inner)[a],
                _p: PhantomData,
            }
        }
    }

    /// Return a mutable iterator.
    #[inline(always)]
    pub fn iter_mut(self) -> PMutIter<'a, T> {
        unsafe {
            PMutIter {
                inner: (*self.inner).iter_mut(),
            }
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

/// Iterator produced by `PMut<[T]>` that generates `PMut<T>`
pub struct PMutIter<'a, T> {
    inner: core::slice::IterMut<'a, T>,
}
impl<'a, T> Iterator for PMutIter<'a, T> {
    type Item = PMut<'a, T>;

    #[inline(always)]
    fn next(&mut self) -> Option<PMut<'a, T>> {
        self.inner.next().map(|inner| PMut {
            inner,
            _p: PhantomData,
        })
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
        self.inner.next_back().map(|inner| PMut {
            inner,
            _p: PhantomData,
        })
    }
}
