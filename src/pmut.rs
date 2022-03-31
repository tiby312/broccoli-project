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
pub struct PMut<T: ?Sized> {
    inner: T,
}

impl<T: std::ops::Deref> core::ops::Deref for PMut<T> {
    type Target = T::Target;
    #[inline(always)]
    fn deref(&self) -> &T::Target {
        &self.inner
    }
}


impl<'a, T> From<&'a mut T> for PMut<&'a mut T> {
    fn from(a: &'a mut T) -> Self {
        PMut::new(a)
    }
}

impl<'a, T: ?Sized> PMut<&'a mut T> {
    /// Create a protected pointer.
    #[inline(always)]
    pub fn new(inner: &'a mut T) -> PMut<&'a mut T> {
        PMut { inner }
    }

    /// Start a new borrow lifetime
    #[inline(always)]
    pub fn borrow_mut(&mut self) -> PMut<&mut T> {
        PMut { inner: self.inner }
    }

    #[inline(always)]
    pub fn into_ref(self) -> &'a T {
        self.inner
    }
}

/// A destructured [`Node`]
pub struct NodeRef<'a, T: Aabb> {
    pub div: &'a Option<T::Num>,
    pub cont: &'a Range<T::Num>,
    pub range: PMut<&'a mut [T]>,
}

impl<'a, 'b: 'a, T: Aabb> PMut<&'a mut Node<'b, T>> {
    /// Destructure a node into its three parts.
    #[inline(always)]
    pub fn into_node_ref(self) -> NodeRef<'a, T> {
        NodeRef {
            div: &self.inner.div,
            cont: &self.inner.cont,
            range: self.inner.range.borrow_mut(),
        }
    }

    #[inline(always)]
    pub fn get_cont(&self) -> &Range<T::Num> {
        &self.cont
    }

    /// Return a mutable list of elements in this node.
    #[inline(always)]
    pub fn into_range(self) -> PMut<&'a mut [T]> {
        self.inner.range.borrow_mut()
    }
}



impl<'a, T: HasInner> PMut<&'a mut T> {
    /// Unpack only the mutable innner component
    #[inline(always)]
    pub fn unpack_inner(self) -> &'a mut T::Inner {
        self.inner.get_inner_mut()
    }
}


impl<'a, T> PMut<&'a mut [T]> {
    /// Return the element at the specified index.
    /// We can't use the index trait because we don't want
    /// to return a mutable reference.
    #[inline(always)]
    pub fn get_index_mut(self, ind: usize) -> PMut<&'a mut T> {
        PMut::new(&mut self.inner[ind])
    }

    /// Split off the first element.
    #[inline(always)]
    pub fn split_at_mut(self, va: usize) -> (PMut<&'a mut [T]>, PMut<&'a mut [T]>) {
        let (left, right) = self.inner.split_at_mut(va);
        (PMut::new(left), PMut::new(right))
    }

    /// Split off the first element.
    #[inline(always)]
    pub fn split_first_mut(self) -> Option<(PMut<&'a mut T>, PMut<&'a mut [T]>)> {
        self.inner
            .split_first_mut()
            .map(|(first, inner)| (PMut { inner: first }, PMut { inner }))
    }

    /// Return a smaller slice that ends with the specified index.
    #[inline(always)]
    pub fn truncate_to(self, a: core::ops::RangeTo<usize>) -> Self {
        PMut {
            inner: &mut self.inner[a],
        }
    }

    /// Return a smaller slice that starts at the specified index.
    #[inline(always)]
    pub fn truncate_from(self, a: core::ops::RangeFrom<usize>) -> Self {
        PMut {
            inner: &mut self.inner[a],
        }
    }

    /// Return a smaller slice that starts and ends with the specified range.
    #[inline(always)]
    pub fn truncate(self, a: core::ops::Range<usize>) -> Self {
        PMut {
            inner: &mut self.inner[a],
        }
    }

    /// Return a mutable iterator.
    #[inline(always)]
    pub fn iter_mut(self) -> PMutIter<'a, T> {
        PMutIter {
            inner: self.inner.iter_mut(),
        }
    }
}

impl<'a, T> core::iter::IntoIterator for PMut<&'a mut [T]> {
    type Item = PMut<&'a mut T>;
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
    type Item = PMut<&'a mut T>;

    #[inline(always)]
    fn next(&mut self) -> Option<PMut<&'a mut T>> {
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
