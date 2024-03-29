use twounordered::TwoUnorderedVecs;

use super::CollisionHandler;
use super::*;

//For sweep and prune type algorithms, we can narrow down which bots
//intersection in one dimension. We also need to check the other direction
//because we know for sure they are colliding. That is the purpose of
//this object.
struct OtherAxisCollider<'a, A: Axis + 'a, F: 'a> {
    a: &'a mut F,
    axis: A,
}

impl<'a, A: Axis + 'a, T: Aabb, F: CollisionHandler<T> + 'a> CollisionHandler<T>
    for OtherAxisCollider<'a, A, F>
{
    #[inline(always)]
    fn collide(&mut self, a: AabbPin<&mut T>, b: AabbPin<&mut T>) {
        //only check if the opoosite axis intersects.
        //already know they intersect
        let a2 = self.axis.next();
        if a.range(a2).intersects(b.range(a2)) {
            self.a.collide(a, b);
        }
    }
}

pub fn sweep_and_prune<'a, A: Axis, T: Aabb, F: CollisionHandler<T>>(
    buffer: &mut Vec<AabbPin<&'a mut T>>,
    axis: A,
    bots: AabbPin<&'a mut [T]>,
    func: &mut F,
) {
    let mut b: OtherAxisCollider<A, _> = OtherAxisCollider { a: func, axis };
    self::find_iter(buffer, axis, bots, &mut b);
}

//Calls colliding on all aabbs that intersect and only one aabbs
//that intsect.
fn find_2d<'a, A: Axis, T: Aabb, F: CollisionHandler<T>>(
    buffer: &mut Vec<AabbPin<&'a mut T>>,
    axis: A,
    bots: AabbPin<&'a mut [T]>,
    func: &mut F,
    check_y: bool,
) {
    if check_y {
        let mut b: OtherAxisCollider<A, _> = OtherAxisCollider { a: func, axis };
        self::find_iter(buffer, axis, bots, &mut b);
    } else {
        let b = func;
        self::find_iter(buffer, axis, bots, b);
    }
}

struct FindParallel2DBuilder<'a, 'b, A: Axis, T: Aabb> {
    pub prevec: &'b mut TwoUnorderedVecs<Vec<AabbPin<&'a mut T>>>,
    pub axis: A,
    pub bots1: AabbPin<&'a mut [T]>,
    pub bots2: AabbPin<&'a mut [T]>,
}

impl<'a, 'b, A: Axis, T: Aabb> FindParallel2DBuilder<'a, 'b, A, T> {
    #[inline(always)]
    pub fn new(
        prevec: &'b mut TwoUnorderedVecs<Vec<AabbPin<&'a mut T>>>,
        axis: A,
        bots1: AabbPin<&'a mut [T]>,
        bots2: AabbPin<&'a mut [T]>,
    ) -> Self {
        FindParallel2DBuilder {
            prevec,
            axis,
            bots1,
            bots2,
        }
    }

    pub fn build(self, mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        self::find_other_parallel4(self.prevec, self.axis, (self.bots1, self.bots2), &mut func);
    }
}

fn find_perp_2d1_once<A: Axis, T: Aabb>(
    axis: A, //the axis of r2.
    mut y: AabbPin<&mut T>,
    mut r2: AabbPin<&mut [T]>,
    mut func: impl CollisionHandler<T>,
) {
    for y2 in r2.borrow_mut() {
        //Exploit the sorted property, to exit early
        if y.range(axis).end < y2.range(axis).start {
            break;
        }

        //Because we didnt exit from the previous comparison, we only need to check one thing.
        if y.range(axis).start <= y2.range(axis).end {
            func.collide(y.borrow_mut(), y2);
        }
    }
}

///Find colliding pairs using the mark and sweep algorithm.
fn find_iter<'a, A: Axis, T: Aabb + 'a, F: CollisionHandler<T>>(
    active: &mut Vec<AabbPin<&'a mut T>>,
    axis: A,
    collision_botids: AabbPin<&'a mut [T]>,
    func: &mut F,
) {
    use twounordered::RetainMutUnordered;
    //    Create a new temporary list called “activeList”.
    //    You begin on the left of your axisList, adding the first item to the activeList.
    //
    //    Now you have a look at the next item in the axisList and compare it with all items
    //     currently in the activeList (at the moment just one):
    //     - If the new item’s left is greater then the current activeList-item right,
    //       then remove
    //    the activeList-item from the activeList
    //     - otherwise report a possible collision between the new axisList-item and the current
    //     activeList-item.
    //
    //    Add the new item itself to the activeList and continue with the next item
    //     in the axisList.

    collision_botids.iter_mut().for_each(|mut curr_bot| {
        active.retain_mut_unordered(|that_bot| {
            let crr = curr_bot.range(axis);

            if that_bot.range(axis).end >= crr.start {
                debug_assert!(curr_bot.range(axis).intersects(that_bot.range(axis)));

                /*
                assert!(curr_bot
                    .get_range(axis.next())
                    .intersects(that_bot.get_range(axis.next())),"{:?} {:?}",curr_bot
                    .get_range(axis.next()),that_bot.get_range(axis.next()));
                */
                func.collide(curr_bot.borrow_mut(), that_bot.borrow_mut());
                true
            } else {
                false
            }
        });

        active.push(curr_bot);
    });
}

/*
#[inline(always)]
fn find_other_parallel3<'a, A: Axis, T: Aabb, F: CollisionHandler<T>>(
    active_lists: &mut TwoUnorderedVecs<Vec<AabbPin<&'a mut T>>>,
    axis: A,
    cols: (AabbPin<&'a mut [T]>, AabbPin<&'a mut [T]>),
    func: &mut F,
) {
    use twounordered::RetainMutUnordered;
    let mut f1 = cols.0.into_iter().peekable();
    let mut f2 = cols.1.into_iter().peekable();

    //Use this to ensure that the active lists
    //are pruned every once in a while even
    //if there are only many many x's in a row with no y's.
    const PRUNE_PERIOD: usize = 100;
    let mut xcounter = 0;
    let mut ycounter = 0;
    loop {
        enum NextP {
            X,
            Y,
        }
        let j = match (f1.peek(), f2.peek()) {
            (Some(_), None) => {
                if active_lists.second().is_empty() {
                    break;
                }

                NextP::X
            }
            (None, Some(_)) => {
                if active_lists.first().is_empty() {
                    break;
                }

                NextP::Y
            }
            (None, None) => {
                break;
            }
            (Some(x), Some(y)) => {
                if x.range(axis).start < y.range(axis).start {
                    NextP::X
                } else {
                    NextP::Y
                }
            }
        };
        match j {
            NextP::X => {
                let mut x = f1.next().unwrap();

                active_lists.second().retain_mut_unordered(|y| {
                    if y.range(axis).end >= x.range(axis).start {
                        func.collide(x.borrow_mut(), y.borrow_mut());
                        true
                    } else {
                        false
                    }
                });
                ycounter = 0;

                if xcounter > PRUNE_PERIOD {
                    active_lists.first().retain_mut_unordered(|x2| {
                        x2.range(axis).end >= x.range(axis).start
                    });
                    xcounter = 0;
                } else {
                    xcounter += 1;
                }

                active_lists.first().push(x);
            }
            NextP::Y => {
                let mut y = f2.next().unwrap();

                active_lists.first().retain_mut_unordered(|x| {
                    if x.range(axis).end >= y.range(axis).start {
                        func.collide(x.borrow_mut(), y.borrow_mut());
                        true
                    } else {
                        false
                    }
                });
                xcounter = 0;

                if ycounter > PRUNE_PERIOD {
                    active_lists.second().retain_mut_unordered(|y2| {
                        y2.range(axis).end >= y.range(axis).start
                    });
                    ycounter = 0;
                } else {
                    ycounter += 1;
                }

                active_lists.second().push(y);
            }
        }
    }
}
*/

#[inline(always)]
#[allow(dead_code)]
fn find_other_parallel4<'a, A: Axis, T: Aabb, F: CollisionHandler<T>>(
    active_lists: &mut TwoUnorderedVecs<Vec<AabbPin<&'a mut T>>>,
    axis: A,
    cols: (AabbPin<&'a mut [T]>, AabbPin<&'a mut [T]>),
    func: &mut F,
) {
    use twounordered::RetainMutUnordered;
    let mut xiter = cols.0.into_iter();
    let mut yiter = cols.1.into_iter();

    //Use this to ensure that the active lists
    //are pruned every once in a while even
    //if there are only many many x's in a row with no y's.
    const PRUNE_PERIOD: usize = 100;
    let mut xcounter = 0;
    let mut ycounter = 0;

    enum NextP<X> {
        X(X),
        Y(X),
    }

    let mut cache: Option<NextP<AabbPin<&mut T>>> = None;
    loop {
        let val = match cache.take() {
            Some(NextP::X(x)) => match yiter.next() {
                Some(y) => {
                    if x.range(axis).start < y.range(axis).start {
                        cache = Some(NextP::Y(y));
                        NextP::X(x)
                    } else {
                        cache = Some(NextP::X(x));
                        NextP::Y(y)
                    }
                }
                None => NextP::X(x),
            },
            Some(NextP::Y(y)) => match xiter.next() {
                Some(x) => {
                    if x.range(axis).start < y.range(axis).start {
                        cache = Some(NextP::Y(y));
                        NextP::X(x)
                    } else {
                        cache = Some(NextP::X(x));
                        NextP::Y(y)
                    }
                }
                None => NextP::Y(y),
            },
            None => match (xiter.next(), yiter.next()) {
                (Some(x), Some(y)) => {
                    if x.range(axis).start < y.range(axis).start {
                        cache = Some(NextP::Y(y));
                        NextP::X(x)
                    } else {
                        cache = Some(NextP::X(x));
                        NextP::Y(y)
                    }
                }
                (Some(x), None) => {
                    if active_lists.second().is_empty() {
                        break;
                    }
                    NextP::X(x)
                }
                (None, Some(y)) => {
                    if active_lists.first().is_empty() {
                        break;
                    }
                    NextP::Y(y)
                }
                (None, None) => {
                    break;
                }
            },
        };

        match val {
            NextP::X(mut x) => {
                active_lists.second().retain_mut_unordered(|y| {
                    if y.range(axis).end >= x.range(axis).start {
                        func.collide(x.borrow_mut(), y.borrow_mut());
                        true
                    } else {
                        false
                    }
                });
                ycounter = 0;

                if xcounter > PRUNE_PERIOD {
                    active_lists
                        .first()
                        .retain_mut_unordered(|x2| x2.range(axis).end >= x.range(axis).start);
                    xcounter = 0;
                } else {
                    xcounter += 1;
                }

                active_lists.first().push(x);
            }
            NextP::Y(mut y) => {
                active_lists.first().retain_mut_unordered(|x| {
                    if x.range(axis).end >= y.range(axis).start {
                        func.collide(x.borrow_mut(), y.borrow_mut());
                        true
                    } else {
                        false
                    }
                });
                xcounter = 0;

                if ycounter > PRUNE_PERIOD {
                    active_lists
                        .second()
                        .retain_mut_unordered(|y2| y2.range(axis).end >= y.range(axis).start);
                    ycounter = 0;
                } else {
                    ycounter += 1;
                }

                active_lists.second().push(y);
            }
        }
    }
}

/* TODO update
#[test]
#[cfg_attr(miri, ignore)]
fn test_parallel() {
    extern crate std;

    use std::collections::BTreeSet;

    #[derive(Copy, Clone, Debug)]
    struct Bot {
        id: usize,
    }

    struct Test {
        set: BTreeSet<[usize; 2]>,
    }
    impl CollisionHandler<BBox<isize, Bot>> for Test {
        fn collide(
            &mut self,
            a: AabbPin<&mut BBox<isize, Bot>>,
            b: AabbPin<&mut BBox<isize, Bot>>,
        ) {
            let [a, b] = [a.unpack_inner().id, b.unpack_inner().id];

            let fin = if a < b { [a, b] } else { [b, a] };
            self.set.insert(fin);
        }
    }

    struct Counter {
        counter: usize,
    }
    impl Counter {
        fn make(&mut self, x1: isize, x2: isize) -> BBox<isize, Bot> {
            let b = BBox::new(rect(x1, x2, 0, 10), Bot { id: self.counter });
            self.counter += 1;
            b
        }
    }

    let mut b = Counter { counter: 0 };

    let mut left = [b.make(0, 10), b.make(5, 20), b.make(10, 40)];
    let mut right = [
        b.make(1, 2),
        b.make(-5, -4),
        b.make(2, 3),
        b.make(-5, -4),
        b.make(3, 4),
        b.make(-5, -4),
        b.make(4, 5),
        b.make(-5, -4),
        b.make(5, 6),
        b.make(-5, -4),
        b.make(6, 7),
    ];

    broccoli_tree::util::sweeper_update(axgeom::XAXIS, &mut left);
    broccoli_tree::util::sweeper_update(axgeom::XAXIS, &mut right);

    let mut p1 = PreVec::new();
    let mut test1 = Test {
        set: BTreeSet::new(),
    };

    let j1: AabbPin<&mut [BBox<_, _>]> = AabbPin::new(&mut left);
    let j2: AabbPin<&mut [BBox<_, _>]> = AabbPin::new(&mut right);

    self::find_other_parallel3(&mut p1, axgeom::XAXIS, (j1, j2), &mut test1);

    let mut test2 = Test {
        set: BTreeSet::new(),
    };
    let j1: AabbPin<&mut [BBox<_, _>]> = AabbPin::new(&mut right);
    let j2: AabbPin<&mut [BBox<_, _>]> = AabbPin::new(&mut left);

    self::find_other_parallel3(&mut p1, axgeom::XAXIS, (j1, j2), &mut test2);

    let diff = test1.set.symmetric_difference(&test2.set);
    let num = diff.clone().count();
    let diff2: Vec<_> = diff.collect();
    assert_eq!(num, 0, "{:?}", &diff2);
}


*/

#[derive(Clone)]
pub struct DefaultNodeHandler<C> {
    pub coll_handler: C,
    prevec: PreVec,
}

impl<C> DefaultNodeHandler<C> {
    pub fn new(coll_handler: C) -> Self {
        DefaultNodeHandler {
            coll_handler,
            prevec: PreVec::new(),
        }
    }
}

impl<T: Aabb, C> NodeHandler<T> for DefaultNodeHandler<C>
where
    C: CollisionHandler<T>,
{
    #[inline(always)]
    fn handle_node(&mut self, axis: AxisDyn, bots: AabbPin<&mut [T]>, is_leaf: bool) {
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
        handle_node(
            &mut self.prevec,
            axis,
            bots,
            &mut self.coll_handler,
            is_leaf,
        );
    }

    fn handle_nodes_under(&mut self, this_axis: AxisDyn, m: VistrMutPin<Node<T, T::Num>>) {
        {
            let (nn, rest) = m.next();

            if let Some([mut left, mut right]) = rest {
                if let Some(div) = nn.div {
                    let d = nn.into_node_ref();
                    let mut g = InnerRecurser {
                        anchor: DNode {
                            div,
                            cont: d.cont,
                            range: d.range,
                        },
                        anchor_axis: this_axis,
                        handler: self,
                    };

                    g.recurse(this_axis.next(), left.borrow_mut(), true);
                    g.recurse(this_axis.next(), right.borrow_mut(), false);
                }
            }
        }
    }
}

impl<'a, T: Aabb> Tree<'a, T> {
    pub fn find_colliding_pairs(&mut self, func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>)) {
        CollisionVisitor::new(self.vistr_mut()).recurse_seq(&mut DefaultNodeHandler::new(func));
    }
}

struct InnerRecurser<'a, T, N, C> {
    anchor: DNode<'a, T, N>,
    anchor_axis: AxisDyn,
    handler: &'a mut DefaultNodeHandler<C>,
}

impl<'a, T: Aabb, C: CollisionHandler<T>> InnerRecurser<'a, T, T::Num, C> {
    fn recurse(&mut self, this_axis: AxisDyn, m: VistrMutPin<Node<T, T::Num>>, is_left: bool) {
        let anchor_axis = self.anchor_axis;

        let (mut nn, rest) = m.next();

        handle_children(
            &mut self.handler.prevec,
            &mut self.handler.coll_handler,
            HandleChildrenArgs {
                anchor: self.anchor.borrow(),
                anchor_axis: self.anchor_axis,
                current: nn.borrow_mut().into_node_ref(),
                current_axis: this_axis,
            },
            is_left,
        );

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

//remove need for second lifetime
struct HandleChildrenArgs<'a, T, N> {
    pub anchor: DNode<'a, T, N>,
    pub current: NodeRef<'a, T, N>,
    pub anchor_axis: AxisDyn,
    pub current_axis: AxisDyn,
}

/// A destructured anchor node.
struct DNode<'a, T, N> {
    pub div: N,
    pub cont: &'a Range<N>,
    pub range: AabbPin<&'a mut [T]>,
}
impl<'a, T, N: Copy> DNode<'a, T, N> {
    fn borrow(&mut self) -> DNode<T, N> {
        DNode {
            div: self.div,
            cont: self.cont,
            range: self.range.borrow_mut(),
        }
    }
}

fn handle_children<T: Aabb, F>(
    prevec: &mut PreVec,
    func: &mut F,
    f: HandleChildrenArgs<T, T::Num>,
    is_left: bool,
) where
    F: CollisionHandler<T>,
{
    fn handle_perp<T: Aabb, A: Axis>(
        axis: A,
        func: &mut impl CollisionHandler<T>,
        f: HandleChildrenArgs<T, T::Num>,
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
                        if a.range(axis).end >= b.range(axis).start {
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
                        if a.range(axis).start <= b.range(axis).end {
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
        f: HandleChildrenArgs<'a, T, T::Num>,
        is_left: bool,
    ) {
        let current2 = f.current;

        let fb =
            oned::FindParallel2DBuilder::new(prevec, axis.next(), f.anchor.range, current2.range);

        if is_left {
            if f.anchor.cont.start <= current2.cont.end {
                fb.build(|a, b| {
                    if a.range(axis).start <= b.range(axis).end {
                        func.collide(a, b)
                    }
                });
            }
        } else if f.anchor.cont.end >= current2.cont.start {
            fb.build(|a, b| {
                if a.range(axis).end >= b.range(axis).start {
                    func.collide(a, b)
                }
            });
        }
    }

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
