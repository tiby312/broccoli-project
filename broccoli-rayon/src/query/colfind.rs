use axgeom::AxisDyn;
use broccoli::{
    aabb::pin::AabbPin,
    aabb::Aabb,
    queries::colfind::{
        build::{CollVis, CollisionHandler, HandleChildrenArgs, NodeHandler},
        handler::AccNodeHandler,
    },
    Tree,
};

use crate::Splitter;

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

        let mut f = AccNodeHandlerEmptySplitter {
            inner: AccNodeHandler::new(floop),
        };

        let vv = CollVis::new(self.vistr_mut());
        recurse_par(vv, &mut f, SEQ_FALLBACK_DEFAULT);
        f.inner.acc.acc
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
        let mut f = AccNodeHandlerEmptySplitter {
            inner: AccNodeHandler::new(FloopDefault { func }),
        };

        let vv = CollVis::new(self.vistr_mut());
        recurse_par(vv, &mut f, num_switch_seq);
    }
}

struct FloopDefault<F> {
    pub func: F,
}
impl<T: Aabb, F> CollisionHandler<T> for FloopDefault<F>
where
    F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
{
    fn collide(&mut self, a: AabbPin<&mut T>, b: AabbPin<&mut T>) {
        (self.func)(a, b)
    }
}
impl<F: Clone> Splitter for FloopDefault<F> {
    fn div(&mut self) -> Self {
        FloopDefault {
            func: self.func.clone(),
        }
    }

    fn add(&mut self, _: Self) {}
}

struct Floop<K, F> {
    acc: K,
    func: F,
}
impl<T: Aabb, K, F> CollisionHandler<T> for Floop<K, F>
where
    F: FnMut(&mut K, AabbPin<&mut T>, AabbPin<&mut T>),
{
    fn collide(&mut self, a: AabbPin<&mut T>, b: AabbPin<&mut T>) {
        (self.func)(&mut self.acc, a, b)
    }
}
impl<K: Splitter, F: Clone> Splitter for Floop<K, F> {
    fn div(&mut self) -> Self {
        let k = self.acc.div();
        Floop {
            acc: k,
            func: self.func.clone(),
        }
    }

    fn add(&mut self, b: Self) {
        self.acc.add(b.acc);
    }
}

struct FloopClosure<K, A, B, F> {
    acc: K,
    div: A,
    add: B,
    func: F,
}
impl<T: Aabb, K, A, B, F> CollisionHandler<T> for FloopClosure<K, A, B, F>
where
    F: FnMut(&mut K, AabbPin<&mut T>, AabbPin<&mut T>),
{
    fn collide(&mut self, a: AabbPin<&mut T>, b: AabbPin<&mut T>) {
        (self.func)(&mut self.acc, a, b)
    }
}
impl<K, A: FnMut(&mut K) -> K + Clone, B: FnMut(&mut K, K) + Clone, F: Clone> Splitter
    for FloopClosure<K, A, B, F>
{
    fn div(&mut self) -> Self {
        FloopClosure {
            acc: (self.div)(&mut self.acc),
            div: self.div.clone(),
            add: self.add.clone(),
            func: self.func.clone(),
        }
    }

    fn add(&mut self, b: Self) {
        (self.add)(&mut self.acc, b.acc)
    }
}

/// Wrapper that impl Splitter
pub struct AccNodeHandlerEmptySplitter<Acc> {
    inner: AccNodeHandler<Acc>,
}

impl<T: Aabb, Acc> NodeHandler<T> for AccNodeHandlerEmptySplitter<Acc>
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
impl<Acc: Splitter> Splitter for AccNodeHandlerEmptySplitter<Acc> {
    fn div(&mut self) -> Self {
        AccNodeHandlerEmptySplitter {
            inner: AccNodeHandler::new(self.inner.acc.div()),
        }
    }

    fn add(&mut self, b: Self) {
        self.inner.acc.add(b.inner.acc);
    }
}

pub fn recurse_par<T: Aabb, SO: NodeHandler<T> + Splitter>(
    vistr: CollVis<T>,
    handler: &mut SO,
    num_seq_fallback: usize,
) where
    T: Send,
    T::Num: Send,
    SO: Send,
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
