//! Provides 2d broadphase collision detection.

mod oned;

use super::tools;
use super::*;
pub mod build;
use build::*;
pub mod handler;
use handler::*;

impl<'a, T: Aabb> Assert<'a, T> {
    ///Panics if a disconnect is detected between all colfind methods.
    pub fn assert_query(&mut self) {
        let bots = &mut self.inner;
        #[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
        pub struct CollisionPtr {
            inner: Vec<(usize, usize)>,
        }

        impl CollisionPtr {
            fn new() -> Self {
                CollisionPtr { inner: vec![] }
            }
            fn add_pair<N>(&mut self, a: &BBox<N, usize>, b: &BBox<N, usize>) {
                let a = a.inner;
                let b = b.inner;
                let (a, b) = if a < b { (a, b) } else { (b, a) };

                self.inner.push((a, b));
            }
            pub fn finish(&mut self) {
                self.inner.sort_unstable();
            }
        }

        let mut bots: Vec<_> = bots
            .iter_mut()
            .enumerate()
            .map(|(i, x)| crate::bbox(*x.get(), i))
            .collect();
        let bots = bots.as_mut_slice();

        let naive_res = {
            let mut cc = CollisionPtr::new();
            Naive::new(bots).find_colliding_pairs(|a, b| {
                cc.add_pair(&*a, &*b);
            });
            cc.finish();
            cc
        };

        let tree_res = {
            let mut cc = CollisionPtr::new();

            Tree::new(bots).find_colliding_pairs(|a, b| {
                cc.add_pair(&*a, &*b);
            });
            cc.finish();
            cc
        };

        let notsort_res = {
            let mut cc = CollisionPtr::new();

            NotSortedTree::new(bots).find_colliding_pairs(|a, b| {
                cc.add_pair(&*a, &*b);
            });
            cc.finish();
            cc
        };

        let sweep_res = {
            let mut cc = CollisionPtr::new();
            SweepAndPrune::new(bots).find_colliding_pairs(|a, b| {
                cc.add_pair(&*a, &*b);
            });
            cc.finish();
            cc
        };

        assert_eq!(naive_res.inner.len(), sweep_res.inner.len());
        assert_eq!(naive_res.inner.len(), tree_res.inner.len());
        assert_eq!(naive_res.inner.len(), notsort_res.inner.len());

        assert_eq!(naive_res, tree_res);
        assert_eq!(naive_res, sweep_res);
        assert_eq!(naive_res, notsort_res);
    }
}

impl<'a, T: Aabb> Naive<'a, T> {
    pub fn find_colliding_pairs(&mut self, mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        queries::for_every_pair(self.inner.borrow_mut(), move |a, b| {
            if a.get().intersects_rect(b.get()) {
                func(a, b);
            }
        });
    }
}

impl<'a, T: Aabb> SweepAndPrune<'a, T> {
    pub fn find_colliding_pairs(&mut self, mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        let mut prevec = Vec::with_capacity(2048);
        let bots = AabbPin::from_mut(self.inner);
        oned::find_2d(&mut prevec, default_axis(), bots, &mut func, true);
    }

    #[cfg(feature = "parallel")]
    ///Sweep and prune algorithm.
    pub fn par_find_colliding_pairs(
        &mut self,
        mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
    ) where
        T: Send,
    {
        let axis = default_axis();
        let mut prevec = Vec::with_capacity(2048);
        let bots = AabbPin::from_mut(self.inner);
        let a2 = axis.next();
        let _ = oned::find_par(
            &mut prevec,
            axis,
            bots,
            move |a: AabbPin<&mut T>, b: AabbPin<&mut T>| {
                if a.get().get_range(a2).intersects(b.get().get_range(a2)) {
                    func(a, b);
                }
            },
        );
    }
}

use crate::tree::splitter::Splitter;

const SEQ_FALLBACK_DEFAULT: usize = 2_400;

impl<'a, T: Aabb> NotSortedTree<'a, T> {
    pub fn find_colliding_pairs(&mut self, func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        BuildArgs::new(NoSortNodeHandler::new(func)).build_ext(CollVis::new(self.vistr_mut()))
    }

    #[cfg(feature = "parallel")]
    pub fn par_find_colliding_pairs<F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>)>(&mut self, func: F)
    where
        T: Send,
        T::Num: Send,
        F: Send + Clone,
    {
        BuildArgs::new(NoSortNodeHandler::new(func)).par_build_ext(CollVis::new(self.vistr_mut()),
        SEQ_FALLBACK_DEFAULT);
    }
}

impl<'a, T: Aabb> Tree<'a, T> {
    pub fn find_colliding_pairs(&mut self, func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        BuildArgs::new(DefaultNodeHandler::new(func)).build_ext(CollVis::new(self.vistr_mut()));
    }

    #[cfg(feature = "parallel")]
    pub fn par_find_colliding_pairs<F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>)>(&mut self, func: F)
    where
        T: Send,
        T::Num: Send,
        F: Send + Clone,
    {
        BuildArgs::new(DefaultNodeHandler::new(func)).par_build_ext(CollVis::new(self.vistr_mut()), SEQ_FALLBACK_DEFAULT)
    }
}

fn recurse_seq<T: Aabb, P: Splitter, SO: NodeHandler<T>>(
    vistr: CollVis<T>,
    splitter: &mut P,
    func: &mut SO,
) {
    let (n, rest) = vistr.collide_and_next(func);

    if let Some([left, right]) = rest {
        let mut s2 = splitter.div();
        n.finish(func);
        recurse_seq(left, splitter, func);
        recurse_seq(right, &mut s2, func);
        splitter.add(s2);
    } else {
        n.finish(func);
    }
}


pub struct BuildArgs<P,SO>{
    splitter:P,
    handler:SO
}

impl<SO> BuildArgs<splitter::EmptySplitter, SO> {
    pub fn new<T:Aabb>(handler:SO)->Self where SO:NodeHandler<T>{
        BuildArgs { splitter: splitter::empty(), handler }
    }
}

impl<P: Splitter, SO> BuildArgs<P, SO> {
    pub fn with_splitter<K:Splitter>(self,splitter:K)->BuildArgs<K,SO>{
        BuildArgs { splitter, handler: self.handler }
    }
    pub fn with_handler<T:Aabb,K:NodeHandler<T>>(self,handler:K)->BuildArgs<P,K>{
        BuildArgs { splitter: self.splitter, handler }
    }

    pub fn build_ext<T:Aabb>(&mut self,vistr:CollVis<T>) where SO:NodeHandler<T> {
        recurse_seq(vistr, &mut self.splitter, &mut self.handler)
    }
    pub fn par_build_ext<T:Aabb>(&mut self,vistr:CollVis<T>,num_seq_fallback: usize)
    where
        SO:NodeHandler<T>,
        T: Send,
        T::Num: Send,
        SO: Splitter + Send,
        P: Send,
    {
        recurse_par(vistr, &mut self.splitter, &mut self.handler, num_seq_fallback);
    }
}


#[cfg(feature = "parallel")]
fn recurse_par<T: Aabb, P: Splitter, SO: NodeHandler<T>>(
    vistr: CollVis<T>,
    splitter: &mut P,
    handler: &mut SO,
    num_seq_fallback: usize,
) where
    T: Send,
    T::Num: Send,
    SO: Splitter + Send,
    P: Send,
{
    if vistr.num_elem() <= num_seq_fallback {
        recurse_seq(vistr, splitter, handler);
    } else {
        let (n, rest) = vistr.collide_and_next(handler);
        if let Some([left, right]) = rest {
            let mut splitter2 = splitter.div();
            let mut h2 = handler.div();

            rayon::join(
                || {
                    n.finish(handler);
                    recurse_par(left, splitter, handler, num_seq_fallback)
                },
                || recurse_par(right, &mut splitter2, &mut h2, num_seq_fallback),
            );
            handler.add(h2);
            splitter.add(splitter2);
        } else {
            n.finish(handler);
        }
    }
}
