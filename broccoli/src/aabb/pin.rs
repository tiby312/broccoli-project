//! Provides a mutable pointer type that is more restrictive that `&mut T`, in order
//! to protect invariants.
//!
//! ```rust
//! use broccoli::{bbox,rect,aabb::pin::AabbPin};
//!
//!
//! let mut a=bbox(rect(0,10,0,10),0);
//! let mut b=bbox(rect(0,10,0,10),0);
//!
//! let ap=AabbPin::new(&mut a);
//! let bp=AabbPin::new(&mut b);
//!
//! //This is not allowed
//! //core::mem::swap(ap,bb);
//!
//! //This is allowed.
//! core::mem::swap(ap.unpack_inner(),bp.unpack_inner());
//!
//!
//! ```

use super::*;

/// Trait exposes an api where you can return a read-only reference to the axis-aligned bounding box
/// and at the same time return a mutable reference to a separate inner section.
pub trait HasInner: Aabb {
    type Inner;

    #[inline(always)]
    fn get_inner_mut(&mut self) -> &mut Self::Inner {
        self.destruct_mut().1
    }

    fn destruct_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner);
}

/// A protected mutable reference that derefs to `&T`.
/// See the AabbPin module documentation for more explanation.
#[repr(transparent)]
#[derive(Debug)]
pub struct AabbPin<T: ?Sized> {
    inner: T,
}

impl<T: std::ops::Deref> core::ops::Deref for AabbPin<T> {
    type Target = T::Target;
    #[inline(always)]
    fn deref(&self) -> &T::Target {
        &self.inner
    }
}

impl<'a, T> From<&'a mut T> for AabbPin<&'a mut T> {
    #[inline(always)]
    fn from(a: &'a mut T) -> Self {
        AabbPin::new(a)
    }
}

impl<T: ?Sized> Clone for AabbPin<*mut T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        AabbPin { inner: self.inner }
    }
}
impl<T: ?Sized> AabbPin<*mut T> {
    #[inline(always)]
    pub fn as_raw(&self) -> *mut T {
        self.inner
    }
}

impl<'a, 'b, T: ?Sized> AabbPin<&'a mut AabbPin<&'b mut T>> {
    #[inline(always)]
    pub fn flatten(self) -> AabbPin<&'a mut T> {
        AabbPin {
            inner: self.inner.inner,
        }
    }
}

impl<'a, T> AabbPin<&'a mut T> {
    pub fn into_slice(self) -> AabbPin<&'a mut [T]> {
        AabbPin {
            inner: std::slice::from_mut(self.inner),
        }
    }
}
impl<'a, T: ?Sized> AabbPin<&'a mut T> {
    /// Start a new borrow lifetime
    #[inline(always)]
    pub fn borrow_mut(&mut self) -> AabbPin<&mut T> {
        AabbPin { inner: self.inner }
    }

    #[inline(always)]
    pub fn as_ptr_mut(&mut self) -> AabbPin<*mut T> {
        AabbPin {
            inner: self.inner as *mut _,
        }
    }

    #[inline(always)]
    pub fn into_ref(self) -> &'a T {
        self.inner
    }
}

impl<'a, T: ?Sized> AabbPin<&'a mut T> {
    /// Create a protected pointer.
    #[inline(always)]
    pub fn from_mut(inner: &'a mut T) -> AabbPin<&'a mut T> {
        AabbPin { inner }
    }
}

impl<T> AabbPin<T> {
    /// Create a protected pointer.
    #[inline(always)]
    pub fn new(inner: T) -> AabbPin<T> {
        AabbPin { inner }
    }
}

/// A destructured [`Node`]
pub struct NodeRef<'a, T, N> {
    pub div: &'a Option<N>,
    pub cont: &'a Range<N>,
    pub range: AabbPin<&'a mut [T]>,
}

impl<'a, 'b: 'a, T, N> AabbPin<&'a mut Node<'b, T, N>> {
    /// Destructure a node into its three parts.
    #[inline(always)]
    pub fn into_node_ref(self) -> NodeRef<'a, T, N> {
        NodeRef {
            div: &self.inner.div,
            cont: &self.inner.cont,
            range: self.inner.range.borrow_mut(),
        }
    }

    #[inline(always)]
    pub fn get_cont(&self) -> &Range<N> {
        &self.cont
    }

    /// Return a mutable list of elements in this node.
    #[inline(always)]
    pub fn into_range(self) -> AabbPin<&'a mut [T]> {
        self.inner.range.borrow_mut()
    }
}

impl<'a, T: HasInner> AabbPin<&'a mut T> {
    #[inline(always)]
    pub fn destruct_mut(&mut self) -> (&Rect<T::Num>, &mut T::Inner) {
        self.inner.destruct_mut()
    }
}

impl<'a, T: HasInner> AabbPin<&'a mut T> {
    /// Unpack only the mutable innner component
    #[inline(always)]
    pub fn unpack_inner(self) -> &'a mut T::Inner {
        self.inner.get_inner_mut()
    }
}

impl<'a, T> AabbPin<&'a mut [T]> {
    /// Return the element at the specified index.
    /// We can't use the index trait because we don't want
    /// to return a mutable reference.
    #[inline(always)]
    pub fn get_index_mut(self, ind: usize) -> AabbPin<&'a mut T> {
        AabbPin::new(&mut self.inner[ind])
    }

    /// Split off the first element.
    #[inline(always)]
    pub fn split_at_mut(self, va: usize) -> (AabbPin<&'a mut [T]>, AabbPin<&'a mut [T]>) {
        let (left, right) = self.inner.split_at_mut(va);
        (AabbPin::new(left), AabbPin::new(right))
    }

    /// Split off the first element.
    #[inline(always)]
    pub fn split_first_mut(self) -> Option<(AabbPin<&'a mut T>, AabbPin<&'a mut [T]>)> {
        self.inner
            .split_first_mut()
            .map(|(first, inner)| (AabbPin { inner: first }, AabbPin { inner }))
    }

    /// Return a smaller slice that ends with the specified index.
    #[inline(always)]
    pub fn truncate_to(self, a: core::ops::RangeTo<usize>) -> Self {
        AabbPin {
            inner: &mut self.inner[a],
        }
    }

    /// Return a smaller slice that starts at the specified index.
    #[inline(always)]
    pub fn truncate_from(self, a: core::ops::RangeFrom<usize>) -> Self {
        AabbPin {
            inner: &mut self.inner[a],
        }
    }

    /// Return a smaller slice that starts and ends with the specified range.
    #[inline(always)]
    pub fn truncate(self, a: core::ops::Range<usize>) -> Self {
        AabbPin {
            inner: &mut self.inner[a],
        }
    }

    /// Return a mutable iterator.
    #[inline(always)]
    pub fn iter_mut(self) -> AabbPinIter<'a, T> {
        AabbPinIter {
            inner: self.inner.iter_mut(),
        }
    }
}

impl<'a, T> core::iter::IntoIterator for AabbPin<&'a mut [T]> {
    type Item = AabbPin<&'a mut T>;
    type IntoIter = AabbPinIter<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// Iterator produced by `AabbPin<[T]>` that generates `AabbPin<T>`
pub struct AabbPinIter<'a, T> {
    inner: core::slice::IterMut<'a, T>,
}
impl<'a, T> Iterator for AabbPinIter<'a, T> {
    type Item = AabbPin<&'a mut T>;

    #[inline(always)]
    fn next(&mut self) -> Option<AabbPin<&'a mut T>> {
        self.inner.next().map(|inner| AabbPin { inner })
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, T> core::iter::FusedIterator for AabbPinIter<'a, T> {}
impl<'a, T> core::iter::ExactSizeIterator for AabbPinIter<'a, T> {}

impl<'a, T> DoubleEndedIterator for AabbPinIter<'a, T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|inner| AabbPin { inner })
    }
}
