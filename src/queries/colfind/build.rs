//!
//! Building blocks to find colliding pairs with trees
//!

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
        match self.axis {
            AxisDyn::X => handler.handle_node(axgeom::XAXIS.next(), self.bots, self.is_leaf),
            AxisDyn::Y => handler.handle_node(axgeom::YAXIS.next(), self.bots, self.is_leaf),
        }
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
        pub struct Recurser<'a, NO> {
            pub handler: &'a mut NO,
        }

        fn collide_self<T: crate::Aabb>(
            this_axis: AxisDyn,
            v: VistrMutPin<Node<T>>,
            data: &mut Recurser<impl NodeHandler<T>>,
        ) {
            let (nn, rest) = v.next();

            if let Some([mut left, mut right]) = rest {
                struct InnerRecurser<'a, 'node, T: Aabb, NN> {
                    anchor: NodeAxis<'a, 'node, T>,
                    handler: &'a mut NN,
                }

                impl<'a, 'node, T: Aabb, NN> InnerRecurser<'a, 'node, T, NN>
                where
                    NN: NodeHandler<T>,
                {
                    fn recurse(
                        &mut self,
                        this_axis: AxisDyn,
                        m: VistrMutPin<Node<T>>,
                        is_left: bool,
                    ) {
                        let anchor_axis = self.anchor.axis;
                        let current_is_leaf = m.get_height() == 1;

                        let (mut nn, rest) = m.next();

                        let current = NodeAxis {
                            node: nn.borrow_mut(),
                            axis: this_axis,
                        };

                        self.handler.handle_children(
                            self.anchor.borrow_mut(),
                            current,
                            current_is_leaf,
                        );

                        if let Some([left, right]) = rest {
                            if let Some(div) = nn.div {
                                if anchor_axis.is_equal_to(this_axis) {
                                    match is_left {
                                        true => {
                                            if div < self.anchor.node.cont.start {
                                                self.recurse(this_axis.next(), right, is_left);
                                                return;
                                            }
                                        }
                                        false => {
                                            if div >= self.anchor.node.cont.end {
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
                        anchor: NodeAxis {
                            node: nn,
                            axis: this_axis,
                        },
                        handler: data.handler,
                    };

                    g.recurse(this_axis.next(), left.borrow_mut(), true);
                    g.recurse(this_axis.next(), right.borrow_mut(), false);
                }
            }
        }

        {
            let mut data = Recurser { handler };

            collide_self(self.axis, self.vistr.borrow_mut(), &mut data);
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

/// Used by [`NodeHandler`]
pub struct NodeAxis<'a, 'node, T: Aabb> {
    pub node: AabbPin<&'a mut Node<'node, T>>,
    pub axis: AxisDyn,
}

impl<'a, 'node, T: Aabb> NodeAxis<'a, 'node, T> {
    #[inline(always)]
    pub fn borrow_mut<'c>(&'c mut self) -> NodeAxis<'c, 'node, T>
    where
        'a: 'c,
    {
        NodeAxis {
            node: self.node.borrow_mut(),
            axis: self.axis,
        }
    }
}

///
/// Abstract over sorted and non sorted trees
///
pub trait NodeHandler<T: Aabb> {
    fn handle_node(&mut self, axis: impl Axis, bots: AabbPin<&mut [T]>, is_leaf: bool);

    fn handle_children(&mut self, anchor: NodeAxis<T>, current: NodeAxis<T>, current_is_leaf: bool);
}

#[derive(Clone)]
pub struct NoSortQuery<F> {
    pub func: F,
}

impl<T: Aabb, F: FnMut(AabbPin<&mut T>, AabbPin<&mut T>)> NodeHandler<T> for NoSortQuery<F> {
    fn handle_node(&mut self, axis: impl Axis, bots: AabbPin<&mut [T]>, is_leaf: bool) {
        if !is_leaf {
            queries::for_every_pair(bots, move |a, b| {
                if a.get().get_range(axis).intersects(b.get().get_range(axis)) {
                    self.func.collide(a, b);
                }
            });
        } else {
            queries::for_every_pair(bots, move |a, b| {
                if a.get().intersects_rect(b.get()) {
                    self.func.collide(a, b);
                }
            });
        }
    }

    fn handle_children(
        &mut self,
        mut anchor: NodeAxis<T>,
        current: NodeAxis<T>,
        _current_is_leaf: bool,
    ) {
        let res = if !current.axis.is_equal_to(anchor.axis) {
            true
        } else {
            current.node.cont.intersects(&anchor.node.cont)
        };

        if res {
            for mut a in current.node.into_range().iter_mut() {
                for mut b in anchor.node.borrow_mut().into_range().iter_mut() {
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
    fn handle_node(&mut self, axis: impl Axis, bots: AabbPin<&mut [T]>, is_leaf: bool) {
        //
        // All bots belonging to a non leaf node are guaranteed to touch the divider.
        // Therefore, all bots intersect along one axis already. Because:
        //
        // If a contains x and b contains x then a intersects b.
        //
        oned::find_2d(&mut self.prevec, axis, bots, &mut self.func, is_leaf);
    }

    #[inline(always)]
    fn handle_children(
        &mut self,
        anchor: NodeAxis<T>,
        current: NodeAxis<T>,
        current_is_leaf: bool,
    ) {
        use AxisDyn::*;
        let func = &mut self.func;
        let prevec = &mut self.prevec;
        match (anchor.axis, current.axis) {
            (X, X) => handle_parallel(XAXIS, prevec, func, anchor, current, current_is_leaf),
            (Y, Y) => handle_parallel(YAXIS, prevec, func, anchor, current, current_is_leaf),
            (X, Y) => handle_perp(XAXIS, func, anchor, current, current_is_leaf),
            (Y, X) => handle_perp(YAXIS, func, anchor, current, current_is_leaf),
        }
    }
}

fn handle_perp<T: Aabb, A: Axis>(
    axis: A,
    func: &mut impl CollisionHandler<T>,
    mut anchor: NodeAxis<T>,
    current: NodeAxis<T>,
    _current_is_leaf: bool,
) {
    let anchor_axis = axis;
    let current_axis = axis.next();

    let cc1 = &anchor.node.cont;
    let div = anchor.node.div.unwrap();

    let cc2 = current.node.into_node_ref();

    //TODO turn into iterator so we dont do two passes
    let r1 = super::tools::get_section_mut(anchor_axis, cc2.range, cc1);

    let mut r2 = super::tools::get_section_mut(
        current_axis,
        anchor.node.borrow_mut().into_range(),
        cc2.cont,
    );

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
fn handle_parallel<T: Aabb, A: Axis>(
    axis: A,
    prevec: &mut PreVec,
    func: &mut impl CollisionHandler<T>,
    anchor: NodeAxis<T>,
    current: NodeAxis<T>,
    current_is_leaf: bool,
) {
    let anchor_div = anchor.node.div.unwrap();

    let anchor2 = anchor.node.into_node_ref();
    let current2 = current.node.into_node_ref();

    //TODO make this even earlier?
    let mut k = twounordered::TwoUnorderedVecs::from(prevec.extract_vec());

    let fb = oned::FindParallel2DBuilder::new(&mut k, axis.next(), anchor2.range, current2.range);

    if current_is_leaf {
        //TODO pointless?
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

    let mut j: Vec<_> = k.into();
    j.clear();
    prevec.insert_vec(j);
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
