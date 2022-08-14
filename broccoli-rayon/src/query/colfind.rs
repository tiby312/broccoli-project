use axgeom::AxisDyn;
use broccoli::{
    queries::colfind::{
        build::{CollVis, CollisionHandler, HandleChildrenArgs, NodeHandler, PreVec},
        AccNodeHandler, FloopDefault,
    },
    tree::{
        aabb_pin::AabbPin,
        node::{Aabb, Node, VistrMutPin},
    },
    Tree,
};

use crate::{EmptySplitter, Splitter};

// struct Floop<K, F> {
//     acc: K,
//     func: F,
// }
// impl<T: Aabb, K, F> CollisionHandler<T> for Floop<K, F>
// where
//     F: FnMut(&mut K, AabbPin<&mut T>, AabbPin<&mut T>),
// {
//     fn collide(&mut self, a: AabbPin<&mut T>, b: AabbPin<&mut T>) {
//         (self.func)(&mut self.acc, a, b)
//     }
// }

pub const SEQ_FALLBACK_DEFAULT: usize = 512;

pub trait RayonQueryPar<'a, T: Aabb> {
    fn par_find_colliding_pairs_ext<F>(&mut self, num_switch_seq: usize, func: F)
    where
        F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
        F: Send + Clone,
        T: Send,
        T::Num: Send;

    fn par_find_colliding_pairs<F>(&mut self, func: F)
    where
        F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
        F: Send + Clone,
        T: Send,
        T::Num: Send;

    fn par_find_colliding_pairs_acc_closure<Acc, A, B, F>(
            &mut self,
            acc: Acc,
            div: A,
            add: B,
            func: F,
        ) -> Acc
        where
            A: FnMut(&mut Acc) -> Acc + Clone + Send,
            B: FnMut(&mut Acc, Acc) + Clone + Send,
            F: FnMut(&mut Acc, AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
            Acc: Send,
            T: Send,
            T::Num: Send;    
}

impl<'a, T: Aabb> RayonQueryPar<'a, T> for Tree<'a, T> {
    fn par_find_colliding_pairs_acc_closure<Acc, A, B, F>(
        &mut self,
        acc: Acc,
        div: A,
        add: B,
        func: F,
    ) -> Acc
    where
        A: FnMut(&mut Acc) -> Acc + Clone + Send,
        B: FnMut(&mut Acc, Acc) + Clone + Send,
        F: FnMut(&mut Acc, AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
        Acc: Send,
        T: Send,
        T::Num: Send,
    {
        let floop = FloopClosure {
            acc,
            div,
            add,
            func,
        };

        let mut f = AccNodeHandler {
            acc: floop,
            prevec: PreVec::new(),
        };
        QueryArgs::new().par_query(self.vistr_mut(), &mut f);
        f.acc.acc
    }


    fn par_find_colliding_pairs<F>(&mut self, func: F)
    where
        F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
        F: Send + Clone,
        T: Send,
        T::Num: Send,
    {
        self.par_find_colliding_pairs_ext(SEQ_FALLBACK_DEFAULT, func);
    }
    fn par_find_colliding_pairs_ext<F>(&mut self, num_switch_seq: usize, func: F)
    where
        F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
        F: Send + Clone,
        T: Send,
        T::Num: Send,
    {
        fn recurse_par<T: Aabb, SO: NodeHandler<T>>(
            vistr: CollVis<T>,
            handler: &mut SO,
            num_seq_fallback: usize,
        ) where
            T: Send,
            T::Num: Send,
            SO: Splitter + Send,
        {
            if vistr.num_elem() <= num_seq_fallback {
                vistr.recurse_seq(handler);
            } else {
                let (n, rest) = vistr.collide_and_next(handler);
                if let Some([left, right]) = rest {
                    let mut h2 = handler.div();

                    rayon::join(
                        || {
                            n.finish(handler);
                            recurse_par(left, handler, num_seq_fallback)
                        },
                        || recurse_par(right, &mut h2, num_seq_fallback),
                    );
                    handler.add(h2);
                } else {
                    n.finish(handler);
                }
            }
        }

        let mut f = AccNodeHandlerWrapper {
            inner: AccNodeHandler {
                acc: FloopDefault { func },
                prevec: PreVec::new(),
            },
            splitter: EmptySplitter,
        };

        let vv = CollVis::new(self.vistr_mut());
        recurse_par(vv, &mut f, num_switch_seq);
    }
}

/// Wrapper that impl Splitter
pub struct AccNodeHandlerWrapper<Acc, S> {
    inner: AccNodeHandler<Acc>,
    splitter: S,
}

impl<T: Aabb, Acc, S> NodeHandler<T> for AccNodeHandlerWrapper<Acc, S>
where
    Acc: CollisionHandler<T>,
{
    #[inline(always)]
    fn handle_node(&mut self, axis: AxisDyn, bots: AabbPin<&mut [T]>, is_leaf: bool) {
        self.inner.handle_node(axis, bots, is_leaf)
    }

    #[inline(always)]
    fn handle_children(&mut self, f: HandleChildrenArgs<T>, is_left: bool) {
        self.inner.handle_children(f, is_left)
    }
}
impl<Acc: Clone, S: Splitter> Splitter for AccNodeHandlerWrapper<Acc, S> {
    fn div(&mut self) -> Self {
        let a = self.splitter.div();
        AccNodeHandlerWrapper {
            inner: self.inner.clone(),
            splitter: a,
        }
    }

    fn add(&mut self, b: Self) {
        self.splitter.add(b.splitter);
    }
}

// pub struct ParQuery {
//     pub num_seq_fallback: usize,
// }
// impl Default for ParQuery {
//     fn default() -> Self {
//         ParQuery {
//             num_seq_fallback: SEQ_FALLBACK_DEFAULT,
//         }
//     }
// }

// impl ParQuery {
//     pub fn par_query<P, T: Aabb, SO>(
//         mut self,
//         splitter: P,
//         vistr: VistrMutPin<Node<T,T::Num>>,
//         handler: &mut SO,
//     ) -> P
//     where
//         P: Splitter,
//         SO: NodeHandler<T>,
//         T: Send,
//         T::Num: Send,
//         SO: Splitter + Send,
//         P: Send,
//     {
//         let vv = CollVis::new(vistr);
//         recurse_par(vv, &mut splitter, handler, self.num_seq_fallback);
//         splitter
//     }

//     #[cfg(feature = "parallel")]
//     pub fn par_find_colliding_pairs_from_args<S: Splitter, F>(
//         &mut self,
//         args: QueryArgs<S>,
//         func: F,
//     ) -> S
//     where
//         F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
//         F: Send + Clone,
//         S: Send,
//         T: Send,
//         T::Num: Send,
//     {
//         let mut f = AccNodeHandler {
//             acc: FloopDefault { func },
//             prevec: PreVec::new(),
//         };
//         args.par_query(self.vistr_mut(), &mut f)
//     }

//     #[cfg(feature = "parallel")]
//     pub fn par_find_colliding_pairs_acc<Acc: Splitter, F>(&mut self, acc: Acc, func: F) -> Acc
//     where
//         F: FnMut(&mut Acc, AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
//         Acc: Splitter + Send,
//         T: Send,
//         T::Num: Send,
//     {
//         let floop = Floop { acc, func };
//         let mut f = AccNodeHandler {
//             acc: floop,
//             prevec: PreVec::new(),
//         };
//         QueryArgs::new().par_query(self.vistr_mut(), &mut f);
//         f.acc.acc
//     }

//     #[cfg(feature = "parallel")]
//     pub fn par_find_colliding_pairs_acc_closure<Acc, A, B, F>(
//         &mut self,
//         acc: Acc,
//         div: A,
//         add: B,
//         func: F,
//     ) -> Acc
//     where
//         A: FnMut(&mut Acc) -> Acc + Clone + Send,
//         B: FnMut(&mut Acc, Acc) + Clone + Send,
//         F: FnMut(&mut Acc, AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
//         Acc: Send,
//         T: Send,
//         T::Num: Send,
//     {
//         let floop = FloopClosure {
//             acc,
//             div,
//             add,
//             func,
//         };

//         let mut f = AccNodeHandler {
//             acc: floop,
//             prevec: PreVec::new(),
//         };
//         QueryArgs::new().par_query(self.vistr_mut(), &mut f);
//         f.acc.acc
//     }
// }
