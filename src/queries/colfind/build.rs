//!
//! Building blocks to find colliding pairs with trees
//!

use twounordered::TwoUnorderedVecs;

use super::*;

///
/// Shorthand for `FnMut(AabbPin<&mut T>, AabbPin<&mut T>)` trait bound
///
pub trait CollisionHandler<T: Aabb> {
    fn collide(&mut self, a: AabbPin<&mut T>, b: AabbPin<&mut T>);
}
impl<T: Aabb, F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>)> CollisionHandler<T> for F {
    #[inline(always)]
    fn collide(&mut self, a: AabbPin<&mut T>, b: AabbPin<&mut T>) {
        self(a, b);
    }
}

///
/// Finish handling a node by calling finish()
///
#[must_use]
pub struct NodeFinisher<'b, T> {
    axis: AxisDyn,
    bots: AabbPin<&'b mut [T]>,
    is_leaf: bool,
}
impl<'b, T: Aabb> NodeFinisher<'b, T> {
    pub fn finish<H: NodeHandler<T>>(self, handler: &mut H) {
        handler.handle_node(self.axis, self.bots, self.is_leaf);
    }
}

/// The main primitive to visit each node and find colliding pairs
pub struct CollVis<'a, 'b, T: Aabb> {
    vistr: VistrMutPin<'b, Node<'a, T>>,
    axis: AxisDyn,
}
impl<'a, 'b, T: Aabb> CollVis<'a, 'b, T> {
    pub(crate) fn new(vistr: VistrMutPin<'b, Node<'a, T>>, axis: AxisDyn) -> Self {
        CollVis { vistr, axis }
    }

    pub fn get_height(&self) -> usize {
        self.vistr.get_height()
    }

    pub fn num_elem(&self) -> usize {
        let (n, _) = self.vistr.borrow().next();
        n.num_elem
    }
    pub fn collide_and_next<N: NodeHandler<T>>(
        mut self,
        handler: &mut N,
    ) -> (NodeFinisher<'b, T>, Option<[Self; 2]>) {
        {
            let this_axis = self.axis;
            let (nn, rest) = self.vistr.borrow_mut().next();

            if let Some([mut left, mut right]) = rest {
                struct InnerRecurser<'a, 'node, T: Aabb, NN> {
                    anchor: AabbPin<&'a mut Node<'node, T>>,
                    anchor_axis: AxisDyn,
                    handler: &'a mut NN,
                }

                impl<'a, 'node, T: Aabb, NN> InnerRecurser<'a, 'node, T, NN>
                where
                    NN: NodeHandler<T>,
                {
                    fn recurse(
                        &mut self,
                        this_axis: AxisDyn,
                        m: VistrMutPin<Node<'node, T>>,
                        is_left: bool,
                    ) {
                        let anchor_axis = self.anchor_axis;
                        let current_is_leaf = m.get_height() == 1;

                        let (mut nn, rest) = m.next();

                        self.handler.handle_children(HandleChildrenArgs {
                            anchor: self.anchor.borrow_mut(),
                            anchor_axis: self.anchor_axis,
                            current: nn.borrow_mut(),
                            current_axis: this_axis,
                            current_is_leaf,
                        });

                        if let Some([left, right]) = rest {
                            if let Some(div) = nn.div {
                                if anchor_axis.is_equal_to(this_axis) {
                                    match is_left {
                                        true => {
                                            if div < self.anchor.cont.start {
                                                self.recurse(this_axis.next(), right, is_left);
                                                return;
                                            }
                                        }
                                        false => {
                                            if div >= self.anchor.cont.end {
                                                self.recurse(this_axis.next(), left, is_left);
                                                return;
                                            }
                                        }
                                    }
                                }
                            }

                            self.recurse(this_axis.next(), left, is_left);
                            self.recurse(this_axis.next(), right, is_left);
                        }
                    }
                }

                if nn.div.is_some() {
                    let mut g = InnerRecurser {
                        anchor: nn,
                        anchor_axis: this_axis,
                        handler,
                    };

                    g.recurse(this_axis.next(), left.borrow_mut(), true);
                    g.recurse(this_axis.next(), right.borrow_mut(), false);
                }
            }
        }

        //TODO make height be zero for leaf?
        let is_leaf = self.get_height() == 1;

        let (n, rest) = self.vistr.next();

        let fin = NodeFinisher {
            axis: self.axis,
            bots: n.into_range(),
            is_leaf,
        };

        (
            fin,
            if let Some([left, right]) = rest {
                Some([
                    CollVis {
                        vistr: left,
                        axis: self.axis.next(),
                    },
                    CollVis {
                        vistr: right,
                        axis: self.axis.next(),
                    },
                ])
            } else {
                None
            },
        )
    }

    pub fn recurse_seq<N: NodeHandler<T>>(self, handler: &mut N) {
        self.recurse_seq_inner(handler)
    }

    fn recurse_seq_inner<N: NodeHandler<T>>(self, handler: &mut N) {
        let (n, rest) = self.collide_and_next(handler);

        n.finish(handler);
        if let Some([a, b]) = rest {
            a.recurse_seq_inner(handler);
            b.recurse_seq_inner(handler);
        }
    }
}

pub struct HandleChildrenArgs<'a, 'node, T: Aabb> {
    pub anchor: AabbPin<&'a mut Node<'node, T>>,
    pub current: AabbPin<&'a mut Node<'node, T>>,
    pub anchor_axis: AxisDyn,
    pub current_axis: AxisDyn,
    pub current_is_leaf: bool,
}

///
/// Abstract over sorted and non sorted trees
///
pub trait NodeHandler<T: Aabb> {
    fn handle_node(&mut self, axis: AxisDyn, bots: AabbPin<&mut [T]>, is_leaf: bool);

    fn handle_children(&mut self, floop: HandleChildrenArgs<T>);
}

#[derive(Clone)]
pub struct NoSortQuery<F> {
    pub func: F,
}

impl<T: Aabb, F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>)> NodeHandler<T> for NoSortQuery<F> {
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
            f.current.cont.intersects(&f.anchor.cont)
        };

        if res {
            for mut a in f.current.into_range().iter_mut() {
                for mut b in f.anchor.borrow_mut().into_range().iter_mut() {
                    if a.get().intersects_rect(b.get()) {
                        self.func.collide(a.borrow_mut(), b.borrow_mut());
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct QueryDefault<F> {
    pub prevec: PreVec,
    pub func: F,
}

impl<T: Aabb, F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>)> NodeHandler<T> for QueryDefault<F> {
    #[inline(always)]
    fn handle_node(&mut self, axis: AxisDyn, bots: AabbPin<&mut [T]>, is_leaf: bool) {
        //
        // All bots belonging to a non leaf node are guaranteed to touch the divider.
        // Therefore, all bots intersect along one axis already. Because:
        //
        // If a contains x and b contains x then a intersects b.
        //
        match axis.next() {
            AxisDyn::X => oned::find_2d(
                &mut self.prevec,
                axgeom::XAXIS,
                bots,
                &mut self.func,
                is_leaf,
            ),
            AxisDyn::Y => oned::find_2d(
                &mut self.prevec,
                axgeom::YAXIS,
                bots,
                &mut self.func,
                is_leaf,
            ),
        }
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

fn handle_perp<T: Aabb, A: Axis>(
    axis: A,
    func: &mut impl CollisionHandler<T>,
    mut f: HandleChildrenArgs<T>,
) {
    let anchor_axis = axis;
    let current_axis = axis.next();

    let cc1 = &f.anchor.cont;
    let div = f.anchor.div.unwrap();

    let cc2 = f.current.into_node_ref();

    //TODO turn into iterator so we dont do two passes
    let r1 = super::tools::get_section_mut(anchor_axis, cc2.range, cc1);

    let mut r2 =
        super::tools::get_section_mut(current_axis, f.anchor.borrow_mut().into_range(), cc2.cont);

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
fn handle_parallel<'a, 'node, T: Aabb, A: Axis>(
    axis: A,
    prevec: &mut TwoUnorderedVecs<Vec<AabbPin<&'a mut T>>>,
    func: &mut impl CollisionHandler<T>,
    f: HandleChildrenArgs<'a, 'node, T>,
) {
    let anchor_div = f.anchor.div.unwrap();

    let anchor2 = f.anchor.into_node_ref();
    let current2 = f.current.into_node_ref();

    let fb = oned::FindParallel2DBuilder::new(prevec, axis.next(), anchor2.range, current2.range);

    if f.current_is_leaf {
        if anchor2.cont.intersects(current2.cont) {
            fb.build(|a, b| {
                if a.get().get_range(axis).intersects(b.get().get_range(axis)) {
                    func.collide(a, b)
                }
            });
        }
    } else if let Some(current_div) = *current2.div {
        if anchor_div < current_div {
            if anchor2.cont.end >= current2.cont.start {
                fb.build(|a, b| {
                    if a.get().get_range(axis).end >= b.get().get_range(axis).start {
                        func.collide(a, b)
                    }
                });
            }
        } else if anchor_div > current_div {
            if anchor2.cont.start <= current2.cont.end {
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

///An vec api to avoid excessive dynamic allocation by reusing a Vec
#[derive(Clone)]
pub struct PreVec {
    vec: Vec<usize>,
}

impl Default for PreVec {
    fn default() -> Self {
        PreVec::new()
    }
}

impl PreVec {
    #[allow(dead_code)]
    #[inline(always)]
    pub fn new() -> PreVec {
        PreVec { vec: Vec::new() }
    }
    #[inline(always)]
    pub fn with_capacity(num: usize) -> PreVec {
        PreVec {
            vec: Vec::with_capacity(num),
        }
    }

    ///Take advantage of the big capacity of the original vec.
    pub fn extract_vec<'a, 'b, T>(&'a mut self) -> Vec<AabbPin<&'b mut T>> {
        let mut v = Vec::new();
        core::mem::swap(&mut v, &mut self.vec);
        revec::convert_empty_vec(v)
    }

    ///Return the big capacity vec
    pub fn insert_vec<T>(&mut self, vec: Vec<AabbPin<&'_ mut T>>) {
        let mut v = revec::convert_empty_vec(vec);
        core::mem::swap(&mut self.vec, &mut v)
    }
}
