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


#[derive(Debug)]
pub struct AabbPin<T> {
    pub(crate) inner: T,
}
impl<T> AabbPin<T>{
    pub fn new(a:T)->Self{
        AabbPin { inner: a }
    }
}

impl<T> std::ops::Deref for AabbPin<&mut T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, T> AabbPin<&'a mut T> {
    pub fn reference(&self) -> &T {
        self.inner
    }
}
impl<'a, T: Unpack> AabbPin<&'a mut T> {
    pub fn unpack(self) -> T::Inner<'a> {
        self.inner.inner()
    }
    pub fn unpack_mut<'b>(&'b mut self) -> T::Inner<'b> {
        self.inner.inner()
    }
}


/// A destructured [`Node`]
pub struct NodeRef<'a, T, N> {
    pub div: &'a Option<N>,
    pub cont: &'a Range<N>,
    pub range: &'a mut [T],
}

impl<'b, T, N> Node<'b, T, N> {
    /// Destructure a node into its three parts.
    #[inline(always)]
    pub fn into_node_ref(&mut self) -> NodeRef<'_, T, N> {
        NodeRef {
            div: &self.div,
            cont: &self.cont,
            range: &mut self.range,
        }
    }

    #[inline(always)]
    pub fn get_cont(&self) -> &Range<N> {
        &self.cont
    }
}

// impl<'a, T> AabbPin<&'a mut [T]> {
//     /// Return the element at the specified index.
//     /// We can't use the index trait because we don't want
//     /// to return a mutable reference.
//     #[inline(always)]
//     pub fn get_index_mut(self, ind: usize) -> AabbPin<&'a mut T> {
//         AabbPin::new(&mut self.inner[ind])
//     }

//     /// Split off the first element.
//     #[inline(always)]
//     pub fn split_at_mut(self, va: usize) -> (AabbPin<&'a mut [T]>, AabbPin<&'a mut [T]>) {
//         let (left, right) = self.inner.split_at_mut(va);
//         (AabbPin::new(left), AabbPin::new(right))
//     }

//     /// Split off the first element.
//     #[inline(always)]
//     pub fn split_first_mut(self) -> Option<(AabbPin<&'a mut T>, AabbPin<&'a mut [T]>)> {
//         self.inner
//             .split_first_mut()
//             .map(|(first, inner)| (AabbPin { inner: first }, AabbPin { inner }))
//     }

//     /// Return a smaller slice that ends with the specified index.
//     #[inline(always)]
//     pub fn truncate_to(self, a: core::ops::RangeTo<usize>) -> Self {
//         AabbPin {
//             inner: &mut self.inner[a],
//         }
//     }

//     /// Return a smaller slice that starts at the specified index.
//     #[inline(always)]
//     pub fn truncate_from(self, a: core::ops::RangeFrom<usize>) -> Self {
//         AabbPin {
//             inner: &mut self.inner[a],
//         }
//     }

//     /// Return a smaller slice that starts and ends with the specified range.
//     #[inline(always)]
//     pub fn truncate(self, a: core::ops::Range<usize>) -> Self {
//         AabbPin {
//             inner: &mut self.inner[a],
//         }
//     }

//     /// Return a mutable iterator.
//     #[inline(always)]
//     pub fn iter_mut(self) -> AabbPinIter<'a, T> {
//         AabbPinIter {
//             inner: self.inner.iter_mut(),
//         }
//     }
// }

// /// Iterator produced by `AabbPin<[T]>` that generates `AabbPin<T>`
// pub struct AabbPinIter<'a, T> {
//     inner: core::slice::IterMut<'a, T>,
// }
// impl<'a, T> Iterator for AabbPinIter<'a, T> {
//     type Item = AabbPin<&'a mut T>;

//     #[inline(always)]
//     fn next(&mut self) -> Option<AabbPin<&'a mut T>> {
//         self.inner.next().map(|inner| AabbPin { inner })
//     }

//     #[inline(always)]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.inner.size_hint()
//     }
// }

// impl<'a, T> core::iter::FusedIterator for AabbPinIter<'a, T> {}
// impl<'a, T> core::iter::ExactSizeIterator for AabbPinIter<'a, T> {}

// impl<'a, T> DoubleEndedIterator for AabbPinIter<'a, T> {
//     #[inline(always)]
//     fn next_back(&mut self) -> Option<Self::Item> {
//         self.inner.next_back().map(|inner| AabbPin { inner })
//     }
// }
