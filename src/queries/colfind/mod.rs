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
            fn add_pair<N>(&mut self, a: &(Rect<N>, usize), b: &(Rect<N>, usize)) {
                let a = a.1;
                let b = b.1;
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
            .map(|(i, x)| (*x.get(), i))
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

use crate::tree::splitter::{EmptySplitter, Splitter};

const SEQ_FALLBACK_DEFAULT: usize = 2_400;

impl<'a, T: Aabb> NotSortedTree<'a, T> {
    pub fn find_colliding_pairs(&mut self, func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        QueryArgs::new().query(self.vistr_mut(), &mut NoSortNodeHandler::new(func));
    }

    #[cfg(feature = "parallel")]
    pub fn par_find_colliding_pairs<F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>)>(&mut self, func: F)
    where
        T: Send,
        T::Num: Send,
        F: Send + Clone,
    {
        let _ = QueryArgs::new().par_query(self.vistr_mut(), &mut NoSortNodeHandler::new(func));
    }
}

impl<'a, T: Aabb> Tree<'a, T> {
    pub fn find_colliding_pairs(&mut self, func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        let mut f=FloopDefault{func};

        let mut f = AccNodeHandler {
            acc:f,
            prevec: PreVec::new(),
        };

        QueryArgs::new().query(self.vistr_mut(), &mut f);
    }

    #[cfg(feature = "parallel")]
    pub fn par_find_colliding_pairs<F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>)>(&mut self, func: F)
    where
        T: Send,
        T::Num: Send,
        F: Send + Clone,
    {
        let mut f=FloopDefault{func};

        let mut f = AccNodeHandler {
            acc:f,
            prevec: PreVec::new(),
        };

        let _ = QueryArgs::new().par_query(self.vistr_mut(), &mut f);
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

pub struct QueryArgs<P> {
    pub num_seq_fallback: usize,
    pub splitter: P,
}

impl Default for QueryArgs<EmptySplitter> {
    fn default() -> Self {
        QueryArgs::new()
    }
}
impl QueryArgs<EmptySplitter> {
    pub fn new() -> Self {
        QueryArgs {
            num_seq_fallback: SEQ_FALLBACK_DEFAULT,
            splitter: EmptySplitter,
        }
    }
}

impl<P: Splitter> QueryArgs<P> {
    pub fn with_splitter<K: Splitter>(self, splitter: K) -> QueryArgs<K> {
        QueryArgs {
            num_seq_fallback: self.num_seq_fallback,
            splitter,
        }
    }
    pub fn with_num_seq_fallback(self, num_seq_fallback: usize) -> Self {
        QueryArgs {
            num_seq_fallback,
            splitter: self.splitter,
        }
    }

    pub fn query<T: Aabb, SO>(mut self, vistr: VistrMutPin<Node<T>>, handler: &mut SO) -> P
    where
        SO: NodeHandler<T>,
    {
        let vv = CollVis::new(vistr);

        recurse_seq(vv, &mut self.splitter, handler);
        self.splitter
    }

    #[cfg(feature = "parallel")]
    pub fn par_query<T: Aabb, SO>(mut self, vistr: VistrMutPin<Node<T>>, handler: &mut SO) -> P
    where
        P: Splitter,
        SO: NodeHandler<T>,
        T: Send,
        T::Num: Send,
        SO: Splitter + Send,
        P: Send,
    {
        let vv = CollVis::new(vistr);

        recurse_par(vv, &mut self.splitter, handler, self.num_seq_fallback);
        self.splitter
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
