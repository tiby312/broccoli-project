//! Provides a mutable pointer type that is more restrictive that `&mut T`, in order
//! to protect tree invariants.
//! [`HalfPin`] is short for protected mutable reference.
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

use super::*;

/// Trait exposes an api where you can return a read-only reference to the axis-aligned bounding box
/// and at the same time return a mutable reference to a separate inner section.
pub trait HasInner {
    type Inner;
    fn get_inner_mut(&mut self) -> &mut Self::Inner;
}

/// A protected mutable reference that derefs to `&T`.
/// See the HalfPin module documentation for more explanation.
#[repr(transparent)]
#[derive(Debug)]
pub struct HalfPin<T: ?Sized> {
    inner: T,
}

impl<T: std::ops::Deref> core::ops::Deref for HalfPin<T> {
    type Target = T::Target;
    #[inline(always)]
    fn deref(&self) -> &T::Target {
        &self.inner
    }
}

impl<'a, T> From<&'a mut T> for HalfPin<&'a mut T> {
    #[inline(always)]
    fn from(a: &'a mut T) -> Self {
        HalfPin::new(a)
    }
}

impl<'a, T: ?Sized> Clone for HalfPin<*mut T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        HalfPin { inner: self.inner }
    }
}
impl<'a, T: ?Sized> HalfPin<*mut T> {
    #[inline(always)]
    pub fn as_raw(&self) -> *mut T {
        self.inner
    }
}

pub struct Escapable<'a, T> {
    pub inner: &'a mut T,
}
impl<'a, T> HasInner for Escapable<'a, T> {
    type Inner = T;
    fn get_inner_mut(&mut self) -> &mut Self::Inner {
        self.inner
    }
}

impl<'a, 'b, T: ?Sized> HalfPin<&'a mut HalfPin<&'b mut T>> {
    #[inline(always)]
    pub fn flatten(self) -> HalfPin<&'a mut T> {
        HalfPin {
            inner: self.inner.inner,
        }
    }
}
impl<'a, T: ?Sized> HalfPin<&'a mut T> {
    /// Start a new borrow lifetime
    #[inline(always)]
    pub fn borrow_mut(&mut self) -> HalfPin<&mut T> {
        HalfPin { inner: self.inner }
    }

    #[inline(always)]
    pub fn as_ptr_mut(&mut self) -> HalfPin<*mut T> {
        HalfPin {
            inner: self.inner as *mut _,
        }
    }

    #[inline(always)]
    pub fn into_ref(self) -> &'a T {
        self.inner
    }
}

impl<'a, T: ?Sized> HalfPin<&'a mut T> {
    /// Create a protected pointer.
    #[inline(always)]
    pub fn from_mut(inner: &'a mut T) -> HalfPin<&'a mut T> {
        HalfPin { inner }
    }
}

impl<T> HalfPin<T> {
    /// Create a protected pointer.
    #[inline(always)]
    pub fn new(inner: T) -> HalfPin<T> {
        HalfPin { inner }
    }
}

/// A destructured [`Node`]
pub struct NodeRef<'a, T: Aabb> {
    pub div: &'a Option<T::Num>,
    pub cont: &'a Range<T::Num>,
    pub range: HalfPin<&'a mut [T]>,
}

impl<'a, 'b: 'a, T: Aabb> HalfPin<&'a mut Node<'b, T>> {
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
    pub fn into_range(self) -> HalfPin<&'a mut [T]> {
        self.inner.range.borrow_mut()
    }
}

impl<'a, T: HasInner> HalfPin<&'a mut T> {
    /// Unpack only the mutable innner component
    #[inline(always)]
    pub fn unpack_inner(self) -> &'a mut T::Inner {
        self.inner.get_inner_mut()
    }
}

impl<'a, T> HalfPin<&'a mut [T]> {
    /// Return the element at the specified index.
    /// We can't use the index trait because we don't want
    /// to return a mutable reference.
    #[inline(always)]
    pub fn get_index_mut(self, ind: usize) -> HalfPin<&'a mut T> {
        HalfPin::new(&mut self.inner[ind])
    }

    /// Split off the first element.
    #[inline(always)]
    pub fn split_at_mut(self, va: usize) -> (HalfPin<&'a mut [T]>, HalfPin<&'a mut [T]>) {
        let (left, right) = self.inner.split_at_mut(va);
        (HalfPin::new(left), HalfPin::new(right))
    }

    /// Split off the first element.
    #[inline(always)]
    pub fn split_first_mut(self) -> Option<(HalfPin<&'a mut T>, HalfPin<&'a mut [T]>)> {
        self.inner
            .split_first_mut()
            .map(|(first, inner)| (HalfPin { inner: first }, HalfPin { inner }))
    }

    /// Return a smaller slice that ends with the specified index.
    #[inline(always)]
    pub fn truncate_to(self, a: core::ops::RangeTo<usize>) -> Self {
        HalfPin {
            inner: &mut self.inner[a],
        }
    }

    /// Return a smaller slice that starts at the specified index.
    #[inline(always)]
    pub fn truncate_from(self, a: core::ops::RangeFrom<usize>) -> Self {
        HalfPin {
            inner: &mut self.inner[a],
        }
    }

    /// Return a smaller slice that starts and ends with the specified range.
    #[inline(always)]
    pub fn truncate(self, a: core::ops::Range<usize>) -> Self {
        HalfPin {
            inner: &mut self.inner[a],
        }
    }

    /// Return a mutable iterator.
    #[inline(always)]
    pub fn iter_mut(self) -> HalfPinIter<'a, T> {
        HalfPinIter {
            inner: self.inner.iter_mut(),
        }
    }
}

impl<'a, T> core::iter::IntoIterator for HalfPin<&'a mut [T]> {
    type Item = HalfPin<&'a mut T>;
    type IntoIter = HalfPinIter<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// Iterator produced by `HalfPin<[T]>` that generates `HalfPin<T>`
pub struct HalfPinIter<'a, T> {
    inner: core::slice::IterMut<'a, T>,
}
impl<'a, T> Iterator for HalfPinIter<'a, T> {
    type Item = HalfPin<&'a mut T>;

    #[inline(always)]
    fn next(&mut self) -> Option<HalfPin<&'a mut T>> {
        self.inner.next().map(|inner| HalfPin { inner })
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, T> core::iter::FusedIterator for HalfPinIter<'a, T> {}
impl<'a, T> core::iter::ExactSizeIterator for HalfPinIter<'a, T> {}

impl<'a, T> DoubleEndedIterator for HalfPinIter<'a, T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|inner| HalfPin { inner })
    }
}
