use crate::{aabb_pin::HasInner, tree::node::Aabb, Tree};

///
/// Used by [`CacheSession::cache_colliding_pairs()`]
///
/// # Safety
///
/// Multiple calls must return the same results while contained in [`CacheSession`]
///
pub unsafe trait TrustedCollisionPairs {
    type T;
    fn for_every_pair(&mut self, func: impl FnMut(&mut Self::T, &mut Self::T));
}

///
/// Used by [`CacheSession::cache_elems()`]
///
/// # Safety
///
/// Multiple calls must return the same results while contained in [`CacheSession`]
///
pub unsafe trait TrustedIterAll {
    type T;
    fn for_every(&mut self, func: impl FnMut(&mut Self::T));
}

///
/// Wrapper around a [`Tree`] that is ready to cache pairs.
///
pub struct IndTree<'a, 'b, T: Aabb>(pub &'b mut Tree<'a, T>);

unsafe impl<T: Aabb + HasInner> TrustedCollisionPairs for IndTree<'_, '_, T> {
    type T = T::Inner;
    fn for_every_pair(&mut self, mut func: impl FnMut(&mut Self::T, &mut Self::T)) {
        self.0.find_colliding_pairs(|a, b| {
            func(a.unpack_inner(), b.unpack_inner());
        })
    }
}

unsafe impl<T: Aabb + HasInner> TrustedIterAll for IndTree<'_, '_, T> {
    type T = T::Inner;
    fn for_every(&mut self, mut func: impl FnMut(&mut Self::T)) {
        for a in self
            .0
            .get_nodes_mut()
            .iter_mut()
            .flat_map(|x| x.into_range())
        {
            func(a.unpack_inner());
        }
    }
}

///
/// Start a caching session.
///
pub struct CacheSession<'a, C> {
    inner: &'a mut C,
}

impl<'a, C> CacheSession<'a, C> {
    pub fn new(a: &'a mut C) -> Self {
        CacheSession { inner: a }
    }
}

///
/// A set of cached elements
///
pub struct FilterCache<'a, C: TrustedIterAll, D> {
    inner: *const CacheSession<'a, C>,
    _p: std::marker::PhantomData<&'a C>,
    data: Vec<(*mut C::T, D)>,
}
unsafe impl<'a, C: TrustedIterAll, D> Send for FilterCache<'a, C, D> {}
unsafe impl<'a, C: TrustedIterAll, D> Sync for FilterCache<'a, C, D> {}

impl<'a, C: TrustedIterAll, D> FilterCache<'a, C, D> {
    pub fn handle(&mut self, c: &mut CacheSession<'a, C>) -> &mut [(&mut C::T, D)] {
        assert_eq!(c as *const _, self.inner);

        let data = self.data.as_mut_slice();
        unsafe { &mut *(data as *mut _ as *mut _) }
    }
}

unsafe impl<'a, C: TrustedCollisionPairs, D> Send for CollidingPairsCache<'a, C, D> {}
unsafe impl<'a, C: TrustedCollisionPairs, D> Sync for CollidingPairsCache<'a, C, D> {}

///
/// A set of cached colliding pairs
///
pub struct CollidingPairsCache<'a, C: TrustedCollisionPairs, D> {
    inner: *const CacheSession<'a, C>,
    _p: std::marker::PhantomData<&'a C>,
    pairs: Vec<(*mut C::T, *mut C::T, D)>,
}

impl<'a, C: TrustedCollisionPairs, D> CollidingPairsCache<'a, C, D> {
    pub fn handle(
        &mut self,
        c: &mut CacheSession<'a, C>,
        mut func: impl FnMut(&mut C::T, &mut C::T, &mut D),
    ) {
        assert_eq!(c as *const _, self.inner);

        for (a, b, c) in self.pairs.iter_mut() {
            let a = unsafe { &mut **a };
            let b = unsafe { &mut **b };
            func(a, b, c);
        }
    }
}
use std::marker::PhantomData;

impl<'a, C: TrustedIterAll> CacheSession<'a, C> {
    pub fn cache_elems<D>(
        &mut self,
        mut func: impl FnMut(&mut C::T) -> Option<D>,
    ) -> FilterCache<'a, C, D> {
        let mut data = vec![];
        self.inner.for_every(|a| {
            if let Some(d) = func(a) {
                data.push((a as *mut _, d));
            }
        });
        FilterCache {
            inner: self as *const _,
            _p: PhantomData,
            data,
        }
    }
}

impl<'a, C: TrustedCollisionPairs> CacheSession<'a, C> {
    pub fn cache_colliding_pairs<D>(
        &mut self,
        mut func: impl FnMut(&mut C::T, &mut C::T) -> Option<D>,
    ) -> CollidingPairsCache<'a, C, D> {
        let mut pairs = vec![];
        self.inner.for_every_pair(|a, b| {
            if let Some(res) = func(a, b) {
                pairs.push((a as *mut _, b as *mut _, res));
            }
        });

        CollidingPairsCache {
            inner: self as *mut _,
            _p: PhantomData,
            pairs,
        }
    }

    pub fn finish(self) -> &'a mut C {
        self.inner
    }
}
