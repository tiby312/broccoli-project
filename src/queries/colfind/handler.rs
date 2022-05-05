use super::*;
use twounordered::TwoUnorderedVecs;
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
        let mut k = self.prevec.extract_vec();
        //
        // All bots belonging to a non leaf node are guaranteed to touch the divider.
        // Therefore, all bots intersect along one axis already. Because:
        //
        // If a contains x and b contains x then a intersects b.
        //
        match axis.next() {
            AxisDyn::X => oned::find_2d(&mut k, axgeom::XAXIS, bots, &mut self.func, is_leaf),
            AxisDyn::Y => oned::find_2d(&mut k, axgeom::YAXIS, bots, &mut self.func, is_leaf),
        }

        k.clear();
        self.prevec.insert_vec(k);
    }

    #[inline(always)]
    fn handle_children(&mut self, f: HandleChildrenArgs<T>) {
        use AxisDyn::*;
        let func = &mut self.func;

        match (f.anchor_axis, f.current_axis) {
            (X, X) | (Y, Y) => {
                let mut k = twounordered::TwoUnorderedVecs::from(self.prevec.extract_vec());

                match f.anchor_axis {
                    X => handle_parallel(XAXIS, &mut k, func, f),
                    Y => handle_parallel(YAXIS, &mut k, func, f),
                }

                let mut j: Vec<_> = k.into();
                j.clear();
                self.prevec.insert_vec(j);
            }
            (X, Y) => handle_perp(XAXIS, func, f),
            (Y, X) => handle_perp(YAXIS, func, f),
        }
    }
}

#[derive(Clone)]
pub struct NoSortNodeHandler<F> {
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

    fn handle_children(&mut self, mut f: HandleChildrenArgs<T>) {
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

pub fn handle_perp<T: Aabb, A: Axis>(
    axis: A,
    func: &mut impl CollisionHandler<T>,
    f: HandleChildrenArgs<T>,
) {
    let anchor_axis = axis;
    let current_axis = axis.next();

    let cc1 = &f.anchor.cont;
    let div = f.anchor.div;

    let cc2 = f.current;

    let r1 = super::tools::get_section_mut(anchor_axis, cc2.range, cc1);

    let mut r2 = super::tools::get_section_mut(current_axis, f.anchor.range, cc2.cont);

    //iterate over current nodes botd
    for y in r1.iter_mut() {
        let r2 = r2.borrow_mut();

        match y.get().get_range(axis).contains_ext(div) {
            std::cmp::Ordering::Equal => {
                oned::find_perp_2d1_once(current_axis, y, r2, |a, b| func.collide(a, b));
            }
            std::cmp::Ordering::Greater => {
                oned::find_perp_2d1_once(current_axis, y, r2, |a, b| {
                    if a.get().get_range(axis).end >= b.get().get_range(axis).start {
                        func.collide(a, b);
                    }
                });
            }
            std::cmp::Ordering::Less => {
                oned::find_perp_2d1_once(current_axis, y, r2, |a, b| {
                    //if a.get().get_range(axis).intersects(b.get().get_range(axis)) {
                    if a.get().get_range(axis).start <= b.get().get_range(axis).end {
                        func.collide(a, b);
                    }
                });
            }
        }

        /*
        oned::find_perp_2d1_once(prevec,current.axis,y,r2,|a,b|{
            if a.get().get_range(axis).intersects(b.get().get_range(axis)) {
            //if a.get().get_range(axis).start<=b.get().get_range(axis).end{
                func.collide(a,b);
            }
        });
        */
    }
}

pub fn handle_parallel<'a, T: Aabb, A: Axis>(
    axis: A,
    prevec: &mut TwoUnorderedVecs<Vec<AabbPin<&'a mut T>>>,
    func: &mut impl CollisionHandler<T>,
    f: HandleChildrenArgs<'a, T>,
) {
    let anchor_div = f.anchor.div;

    //let anchor2 = f.anchor.into_node_ref();
    let current2 = f.current;

    let fb = oned::FindParallel2DBuilder::new(prevec, axis.next(), f.anchor.range, current2.range);

    if f.current_is_leaf {
        if f.anchor.cont.intersects(current2.cont) {
            fb.build(|a, b| {
                if a.get().get_range(axis).intersects(b.get().get_range(axis)) {
                    func.collide(a, b)
                }
            });
        }
    } else if let Some(current_div) = *current2.div {
        if anchor_div < current_div {
            if f.anchor.cont.end >= current2.cont.start {
                fb.build(|a, b| {
                    if a.get().get_range(axis).end >= b.get().get_range(axis).start {
                        func.collide(a, b)
                    }
                });
            }
        } else if anchor_div > current_div {
            if f.anchor.cont.start <= current2.cont.end {
                fb.build(|a, b| {
                    if a.get().get_range(axis).start <= b.get().get_range(axis).end {
                        func.collide(a, b)
                    }
                });
            }
        } else {
            fb.build(|a, b| {
                func.collide(a, b);
            });
        }
    }
}
