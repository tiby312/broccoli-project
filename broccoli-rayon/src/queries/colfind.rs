use broccoli::{
    aabb::pin::AabbPin,
    aabb::Aabb,
    queries::colfind::{
        build::{CollisionHandler, CollisionVisitor, NodeHandler},
        oned::DefaultNodeHandler,
    },
    Tree,
};

pub trait CollisionHandlerExt<T: Aabb>: CollisionHandler<T> + Sized {
    ///Called to split this into two to be passed to the children.
    fn div(&mut self) -> Self;

    ///Called to add the results of the recursive calls on the children.
    fn add(&mut self, b: Self);
}

pub trait NodeHandlerExt<T: Aabb>: NodeHandler<T> + Sized {
    ///Called to split this into two to be passed to the children.
    fn div(&mut self) -> Self;

    ///Called to add the results of the recursive calls on the children.
    fn add(&mut self, b: Self);
}

//pub const SEQ_FALLBACK_DEFAULT: usize = 512;
pub const SEQ_FALLBACK_DEFAULT: usize = 256;

pub trait RayonQueryPar<'a, T: Aabb> {
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
        let mut f = DefaultNodeHandler::new(ClosureExt {
            acc,
            div,
            add,
            func,
        });

        let vv = CollisionVisitor::new(self.vistr_mut());
        recurse_par(vv, &mut f, SEQ_FALLBACK_DEFAULT);
        f.coll_handler.acc
    }

    fn par_find_colliding_pairs<F>(&mut self, func: F)
    where
        F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>) + Clone,
        F: Send,
        T: Send,
        T::Num: Send,
    {
        let mut f = DefaultNodeHandler::new(func);

        let vv = CollisionVisitor::new(self.vistr_mut());
        recurse_par(vv, &mut f, SEQ_FALLBACK_DEFAULT);
    }
}

impl<F, T: Aabb> CollisionHandlerExt<T> for F
where
    F: Clone + FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
{
    fn div(&mut self) -> Self {
        self.clone()
    }

    fn add(&mut self, _: Self) {}
}

///
/// Collision call back handler that has callbacks
/// to handle the events where the closure has to be split
/// off and then joined again.
///
pub struct ClosureExt<K, A, B, F> {
    pub acc: K,
    pub div: A,
    pub add: B,
    pub func: F,
}
impl<T: Aabb, K, A, B, F> CollisionHandler<T> for ClosureExt<K, A, B, F>
where
    F: FnMut(&mut K, AabbPin<&mut T>, AabbPin<&mut T>),
{
    fn collide(&mut self, a: AabbPin<&mut T>, b: AabbPin<&mut T>) {
        (self.func)(&mut self.acc, a, b)
    }
}
impl<T: Aabb, K, A: FnMut(&mut K) -> K + Clone, B: FnMut(&mut K, K) + Clone, F: Clone>
    CollisionHandlerExt<T> for ClosureExt<K, A, B, F>
where
    F: FnMut(&mut K, AabbPin<&mut T>, AabbPin<&mut T>),
{
    fn div(&mut self) -> Self {
        ClosureExt {
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

impl<Acc: CollisionHandlerExt<T>, T: Aabb> NodeHandlerExt<T> for DefaultNodeHandler<Acc> {
    fn div(&mut self) -> Self {
        DefaultNodeHandler::new(self.coll_handler.div())
    }

    fn add(&mut self, b: Self) {
        self.coll_handler.add(b.coll_handler);
    }
}

pub fn recurse_par<T: Aabb, SO: NodeHandlerExt<T>>(
    vistr: CollisionVisitor<T>,
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
