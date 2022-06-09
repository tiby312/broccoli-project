use super::*;
use twounordered::TwoUnorderedVecs;

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

impl<T: Aabb> Tree<'_, T> {
    pub fn find_colliding_pairs_from_args<S: Splitter>(
        &mut self,
        args: QueryArgs<S>,
        func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
    ) -> S {
        let mut f = AccNodeHandler {
            acc: FloopDefault { func },
            prevec: PreVec::new(),
        };
        args.query(self.vistr_mut(), &mut f)
    }

    #[cfg(feature = "parallel")]
    pub fn par_find_colliding_pairs_from_args<S: Splitter, F>(
        &mut self,
        args: QueryArgs<S>,
        func: F,
    ) -> S
    where
        F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
        F: Send + Clone,
        S: Send,
        T: Send,
        T::Num: Send,
    {
        let mut f = AccNodeHandler {
            acc: FloopDefault { func },
            prevec: PreVec::new(),
        };
        args.par_query(self.vistr_mut(), &mut f)
    }

    #[cfg(feature = "parallel")]
    pub fn par_find_colliding_pairs_acc<Acc: Splitter, F>(&mut self, acc: Acc, func: F) -> Acc
    where
        F: FnMut(&mut Acc, AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
        Acc: Splitter + Send,
        T: Send,
        T::Num: Send,
    {
        let floop = Floop { acc, func };
        let mut f = AccNodeHandler {
            acc: floop,
            prevec: PreVec::new(),
        };
        QueryArgs::new().par_query(self.vistr_mut(), &mut f);
        f.acc.acc
    }

    #[cfg(feature = "parallel")]
    pub fn par_find_colliding_pairs_acc_closure<Acc, A, B, F>(
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

struct AccNodeHandler<Acc> {
    pub acc: Acc,
    pub prevec: PreVec,
}

impl<Acc: Splitter> Splitter for AccNodeHandler<Acc> {
    fn div(&mut self) -> Self {
        let acc = self.acc.div();

        AccNodeHandler {
            acc,
            prevec: self.prevec.clone(),
        }
    }

    fn add(&mut self, b: Self) {
        self.acc.add(b.acc);
    }
}

impl<T: Aabb, Acc> NodeHandler<T> for AccNodeHandler<Acc>
where
    Acc: CollisionHandler<T>,
{
    #[inline(always)]
    fn handle_node(&mut self, axis: AxisDyn, bots: AabbPin<&mut [T]>, is_leaf: bool) {
        handle_node(&mut self.prevec, axis, bots, &mut self.acc, is_leaf);
    }

    #[inline(always)]
    fn handle_children(&mut self, f: HandleChildrenArgs<T>, is_left: bool) {
        handle_children(&mut self.prevec, &mut self.acc, f, is_left)
    }
}

impl<'a, T: Aabb> Tree<'a, T> {
    pub fn find_colliding_pairs(&mut self, func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        let mut f = AccNodeHandler {
            acc: FloopDefault { func },
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
        let mut f = AccNodeHandler {
            acc: FloopDefault { func },
            prevec: PreVec::new(),
        };

        let _ = QueryArgs::new().par_query(self.vistr_mut(), &mut f);
    }
}

/*
#[derive(Clone)]
pub struct DefaultNodeHandler<F> {
    pub prevec: PreVec,
    pub func: F,
}

impl<F> DefaultNodeHandler<F> {
    pub fn new<T: Aabb>(func: F) -> Self
    where
        F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
    {
        DefaultNodeHandler {
            func,
            prevec: PreVec::new(),
        }
    }
}

impl<F: Clone> Splitter for DefaultNodeHandler<F> {
    fn div(&mut self) -> Self {
        DefaultNodeHandler {
            prevec: self.prevec.clone(),
            func: self.func.clone(),
        }
    }

    fn add(&mut self, _b: Self) {}
}

impl<T: Aabb, F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>)> NodeHandler<T> for DefaultNodeHandler<F> {
    #[inline(always)]
    fn handle_node(&mut self, axis: AxisDyn, bots: AabbPin<&mut [T]>, is_leaf: bool) {
        handle_node(&mut self.prevec, axis, bots, &mut self.func, is_leaf);
    }

    #[inline(always)]
    fn handle_children(&mut self, f: HandleChildrenArgs<T>) {
        handle_children(&mut self.prevec, &mut self.func, f)
    }
}
*/

fn handle_node<T: Aabb, F>(
    prevec: &mut PreVec,
    axis: AxisDyn,
    bots: AabbPin<&mut [T]>,
    func: &mut F,
    is_leaf: bool,
) where
    F: CollisionHandler<T>,
{
    let mut k = prevec.extract_vec();
    //
    // All bots belonging to a non leaf node are guaranteed to touch the divider.
    // Therefore, all bots intersect along one axis already. Because:
    //
    // If a contains x and b contains x then a intersects b.
    //
    match axis.next() {
        AxisDyn::X => oned::find_2d(&mut k, axgeom::XAXIS, bots, func, is_leaf),
        AxisDyn::Y => oned::find_2d(&mut k, axgeom::YAXIS, bots, func, is_leaf),
    }

    k.clear();
    prevec.insert_vec(k);
}

fn handle_children<T: Aabb, F>(
    prevec: &mut PreVec,
    func: &mut F,
    f: HandleChildrenArgs<T>,
    is_left: bool,
) where
    F: CollisionHandler<T>,
{
    use AxisDyn::*;

    match (f.anchor_axis, f.current_axis) {
        (X, X) | (Y, Y) => {
            let mut k = twounordered::TwoUnorderedVecs::from(prevec.extract_vec());

            match f.anchor_axis {
                X => handle_parallel(XAXIS, &mut k, func, f, is_left),
                Y => handle_parallel(YAXIS, &mut k, func, f, is_left),
            }

            let mut j: Vec<_> = k.into();
            j.clear();
            prevec.insert_vec(j);
        }
        (X, Y) => handle_perp(XAXIS, func, f, is_left),
        (Y, X) => handle_perp(YAXIS, func, f, is_left),
    }
}
impl<T: Aabb> NotSortedTree<'_, T> {
    pub fn find_colliding_pairs_from_args<S: Splitter>(
        &mut self,
        args: QueryArgs<S>,
        func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
    ) -> S {
        args.query(self.vistr_mut(), &mut NoSortNodeHandler::new(func))
    }

    #[cfg(feature = "parallel")]
    pub fn par_find_colliding_pairs_from_args<S: Splitter, F>(
        &mut self,
        args: QueryArgs<S>,
        func: F,
    ) -> S
    where
        F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
        F: Send + Clone,
        S: Send,
        T: Send,
        T::Num: Send,
    {
        args.par_query(self.vistr_mut(), &mut NoSortNodeHandler::new(func))
    }
}

#[derive(Clone)]
struct NoSortNodeHandler<F> {
    pub func: F,
}
impl<F> NoSortNodeHandler<F> {
    pub fn new<T: Aabb>(func: F) -> Self
    where
        F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
    {
        NoSortNodeHandler { func }
    }
}

impl<F: Clone> Splitter for NoSortNodeHandler<F> {
    fn div(&mut self) -> Self {
        NoSortNodeHandler {
            func: self.func.clone(),
        }
    }

    fn add(&mut self, _b: Self) {}
}

impl<T: Aabb, F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>)> NodeHandler<T> for NoSortNodeHandler<F> {
    fn handle_node(&mut self, axis: AxisDyn, bots: AabbPin<&mut [T]>, is_leaf: bool) {
        fn foop<T: Aabb, F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>)>(
            func: &mut F,
            axis: impl Axis,
            bots: AabbPin<&mut [T]>,
            is_leaf: bool,
        ) {
            if !is_leaf {
                queries::for_every_pair(bots, move |a, b| {
                    if a.get().get_range(axis).intersects(b.get().get_range(axis)) {
                        func.collide(a, b);
                    }
                });
            } else {
                queries::for_every_pair(bots, move |a, b| {
                    if a.get().intersects_rect(b.get()) {
                        func.collide(a, b);
                    }
                });
            }
        }

        match axis.next() {
            AxisDyn::X => foop(&mut self.func, XAXIS, bots, is_leaf),
            AxisDyn::Y => foop(&mut self.func, YAXIS, bots, is_leaf),
        }
    }

    fn handle_children(&mut self, mut f: HandleChildrenArgs<T>, _is_left: bool) {
        let res = if !f.current_axis.is_equal_to(f.anchor_axis) {
            true
        } else {
            f.current.cont.intersects(f.anchor.cont)
        };

        if res {
            for mut a in f.current.range.iter_mut() {
                for mut b in f.anchor.range.borrow_mut().iter_mut() {
                    if a.get().intersects_rect(b.get()) {
                        self.func.collide(a.borrow_mut(), b.borrow_mut());
                    }
                }
            }
        }
    }
}

fn handle_perp<T: Aabb, A: Axis>(
    axis: A,
    func: &mut impl CollisionHandler<T>,
    f: HandleChildrenArgs<T>,
    is_left: bool,
) {
    let anchor_axis = axis;
    let current_axis = axis.next();

    let cc1 = f.anchor.cont;

    let cc2 = f.current;

    let r1 = super::tools::get_section_mut(anchor_axis, cc2.range, cc1);

    let mut r2 = super::tools::get_section_mut(current_axis, f.anchor.range, cc2.cont);

    if is_left {
        //iterate over current nodes botd
        for y in r1.iter_mut() {
            let r2 = r2.borrow_mut();

            oned::find_perp_2d1_once(
                current_axis,
                y,
                r2,
                |a: AabbPin<&mut T>, b: AabbPin<&mut T>| {
                    if a.get().get_range(axis).end >= b.get().get_range(axis).start {
                        func.collide(a, b);
                    }
                },
            );
        }
    } else {
        //iterate over current nodes botd
        for y in r1.iter_mut() {
            let r2 = r2.borrow_mut();

            oned::find_perp_2d1_once(
                current_axis,
                y,
                r2,
                |a: AabbPin<&mut T>, b: AabbPin<&mut T>| {
                    if a.get().get_range(axis).start <= b.get().get_range(axis).end {
                        func.collide(a, b);
                    }
                },
            );
        }
    }
}

fn handle_parallel<'a, T: Aabb, A: Axis>(
    axis: A,
    prevec: &mut TwoUnorderedVecs<Vec<AabbPin<&'a mut T>>>,
    func: &mut impl CollisionHandler<T>,
    f: HandleChildrenArgs<'a, T>,
    is_left: bool,
) {
    let current2 = f.current;

    let fb = oned::FindParallel2DBuilder::new(prevec, axis.next(), f.anchor.range, current2.range);

    if is_left {
        if f.anchor.cont.start <= current2.cont.end {
            fb.build(|a, b| {
                if a.get().get_range(axis).start <= b.get().get_range(axis).end {
                    func.collide(a, b)
                }
            });
        }
    } else if f.anchor.cont.end >= current2.cont.start {
        fb.build(|a, b| {
            if a.get().get_range(axis).end >= b.get().get_range(axis).start {
                func.collide(a, b)
            }
        });
    }
}
