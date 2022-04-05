use std::ops::DerefMut;

use broccoli::{
    prelude::CollisionApi,
    tree::{treepin::HasInner, node::Aabb, Tree},
};


pub unsafe trait TrustedCollisionPairs<T> {
    fn for_every_pair(&mut self, func: impl FnMut(&mut T, &mut T));
}

unsafe impl<'a, T: Aabb + HasInner<Inner = K>, K: DerefMut> TrustedCollisionPairs<K::Target>
    for Tree<'a, T>
where
    K::Target: Sized,
{
    fn for_every_pair(&mut self, mut func: impl FnMut(&mut K::Target, &mut K::Target)) {
        self.colliding_pairs(|a, b| {
            func(a.unpack_inner(), b.unpack_inner());
        })
    }
}

pub struct Cacheable<'a, T, C, D> {
    inner: &'a mut C,
    pairs: Vec<(*mut T, *mut T, D)>,
}
impl<'a, T, C: TrustedCollisionPairs<T>> Cacheable<'a, T, C, ()> {
    pub fn new(a: &'a mut C) -> Self {
        Self::with_func(a, |_, _| Some(()))
    }
}

impl<'a, T, C: TrustedCollisionPairs<T>, D> Cacheable<'a, T, C, D> {
    pub fn with_func(a: &'a mut C, mut func: impl FnMut(&mut T, &mut T) -> Option<D>) -> Self {
        let mut pairs = vec![];
        a.for_every_pair(|a, b| {
            if let Some(res) = func(a, b) {
                pairs.push((a as *mut _, b as *mut _, res));
            }
        });

        Cacheable { inner: a, pairs }
    }

    pub fn colliding_pairs(&mut self, mut func: impl FnMut(&mut T, &mut T, &mut D)) {
        for (a, b, c) in self.pairs.iter_mut() {
            let a = unsafe { &mut **a };
            let b = unsafe { &mut **b };
            func(a, b, c);
        }
    }

    pub fn finish(self) -> &'a mut C {
        self.inner
    }
}
