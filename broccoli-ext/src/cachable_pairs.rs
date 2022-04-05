use std::ops::DerefMut;

use broccoli::{
    prelude::CollisionApi,
    tree::{treepin::HasInner, node::Aabb, Tree},
};


pub unsafe trait TrustedCollisionPairs {
    type T;
    fn for_every_pair(&mut self, func: impl FnMut(&mut Self::T, &mut Self::T));
}

unsafe impl<'a, T: Aabb + HasInner<Inner = K>, K: DerefMut> TrustedCollisionPairs
    for Tree<'a, T>
where
    K::Target: Sized,
{
    type T=K::Target;
    fn for_every_pair(&mut self, mut func: impl FnMut(&mut K::Target, &mut K::Target)) {
        self.colliding_pairs(|a, b| {
            func(a.unpack_inner(), b.unpack_inner());
        })
    }
}



pub struct Cacheable<'a, C> {
    inner: &'a mut C,
}

impl<'a, C> Cacheable<'a, C> {
    pub fn new(a: &'a mut C) -> Self {
        Cacheable{
            inner:a
        }
    }
}


pub struct RectCache<'a,C:TrustedCollisionPairs,D>{
    inner: *mut Cacheable<'a,C>,
    _p:std::marker::PhantomData<&'a C>,
    pairs:Vec<(*mut C::T,D)>
}

pub struct CollidingPairsCache<'a,C:TrustedCollisionPairs,D>{
    inner: *mut Cacheable<'a,C>,
    _p:std::marker::PhantomData<&'a C>,
    pairs:Vec<(*mut C::T,*mut C::T,D)>
}

impl<'a,C:TrustedCollisionPairs,D> CollidingPairsCache<'a,C,D>{
    pub fn colliding_pairs(&mut self,c:&mut Cacheable<'a,C>, mut func: impl FnMut(&mut C::T, &mut C::T, &mut D)) {
        assert_eq!(c as *mut _,self.inner);

        for (a, b, c) in self.pairs.iter_mut() {
            let a = unsafe { &mut **a };
            let b = unsafe { &mut **b };
            func(a, b, c);
        }
    }
}
use std::marker::PhantomData;


impl<'a, C: TrustedCollisionPairs> Cacheable<'a, C> {
    pub fn cache_colliding_pairs<D>(&mut self, mut func: impl FnMut(&mut C::T, &mut C::T) -> Option<D>) -> CollidingPairsCache<'a,C,D> {
        let mut pairs = vec![];
        self.inner.for_every_pair(|a, b| {
            if let Some(res) = func(a, b) {
                pairs.push((a as *mut _, b as *mut _, res));
            }
        });

        CollidingPairsCache { inner: self as *mut _, _p: PhantomData, pairs }
    }

    pub fn cache_rect<D>(&mut self,mut func:impl FnMut(&mut C::T,&mut C::T)->Option<D>)->RectCache<'a,C,D>{
        unimplemented!();
    }

    

    pub fn finish(self) -> &'a mut C {
        self.inner
    }
}
