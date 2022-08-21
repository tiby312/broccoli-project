use broccoli::{
    aabb::pin::AabbPin,
    aabb::Aabb,
    queries::colfind::{
        build::{CollVis, CollisionHandler, NodeHandler},
        handler::DefaultNodeHandler,
    },
    Tree,
};

///A trait that gives the user callbacks at events in a recursive algorithm on the tree.
///The main motivation behind this trait was to track the time spent taken at each level of the tree
///during construction.
pub trait Splitter: Sized {
    ///Called to split this into two to be passed to the children.
    fn div(&mut self) -> Self;

    ///Called to add the results of the recursive calls on the children.
    fn add(&mut self, b: Self);
}

pub struct EmptySplitter;

impl Splitter for EmptySplitter {
    fn div(&mut self) -> Self {
        EmptySplitter
    }
    fn add(&mut self, _: Self) {}
}

//pub const SEQ_FALLBACK_DEFAULT: usize = 512;
pub const SEQ_FALLBACK_DEFAULT: usize = 256;

pub trait RayonQueryPar<'a, T: Aabb> {
    // fn par_find_colliding_pairs_ext<F>(&mut self, num_switch_seq: usize, func: F)
    // where
    //     F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
    //     F: Send + Clone,
    //     T: Send,
    //     T::Num: Send;

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
        let floop = ClosureSplitter {
            acc,
            div,
            add,
            func,
        };

        let mut f = DefaultNodeHandler::new(floop);

        let vv = CollVis::new(self.vistr_mut());
        recurse_par(vv, &mut f, SEQ_FALLBACK_DEFAULT);
        f.acc.acc
    }

    fn par_find_colliding_pairs<F>(&mut self, func: F)
    where
        F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>) + Clone,
        F: Send,
        T: Send,
        T::Num: Send,
    {
        let mut f = DefaultNodeHandler::new(ClosureCloneable { func });

        let vv = CollVis::new(self.vistr_mut());
        recurse_par(vv, &mut f, SEQ_FALLBACK_DEFAULT);
        //self.par_find_colliding_pairs_ext(SEQ_FALLBACK_DEFAULT, func);
    }
}

///
/// collision callback handler that is cloneable.
///
pub struct ClosureCloneable<F> {
    pub func: F,
}
impl<T: Aabb, F> CollisionHandler<T> for ClosureCloneable<F>
where
    F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
{
    fn collide(&mut self, a: AabbPin<&mut T>, b: AabbPin<&mut T>) {
        (self.func)(a, b)
    }
}
impl<F: Clone> Splitter for ClosureCloneable<F> {
    fn div(&mut self) -> Self {
        ClosureCloneable {
            func: self.func.clone(),
        }
    }

    fn add(&mut self, _: Self) {}
}

///
/// Collision call back handler that has callbacks
/// to handle the events where the closure has to be split
/// off and then joined again.
///
pub struct ClosureSplitter<K, A, B, F> {
    pub acc: K,
    pub div: A,
    pub add: B,
    pub func: F,
}
impl<T: Aabb, K, A, B, F> CollisionHandler<T> for ClosureSplitter<K, A, B, F>
where
    F: FnMut(&mut K, AabbPin<&mut T>, AabbPin<&mut T>),
{
    fn collide(&mut self, a: AabbPin<&mut T>, b: AabbPin<&mut T>) {
        (self.func)(&mut self.acc, a, b)
    }
}
impl<K, A: FnMut(&mut K) -> K + Clone, B: FnMut(&mut K, K) + Clone, F: Clone> Splitter
    for ClosureSplitter<K, A, B, F>
{
    fn div(&mut self) -> Self {
        ClosureSplitter {
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

impl<Acc: Splitter> Splitter for DefaultNodeHandler<Acc> {
    fn div(&mut self) -> Self {
        DefaultNodeHandler::new(self.acc.div())
    }

    fn add(&mut self, b: Self) {
        self.acc.add(b.acc);
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
