use super::super::ColMulti;
use crate::query::inner_prelude::*;

//For sweep and prune type algorithms, we can narrow down which bots
//intersection in one dimension. We also need to check the other direction
//because we know for sure they are colliding. That is the purpose of
//this object.
struct OtherAxisCollider<'a, A: Axis + 'a, F: ColMulti + 'a> {
    a: &'a mut F,
    axis: A,
}

impl<'a, A: Axis + 'a, F: ColMulti + 'a> ColMulti for OtherAxisCollider<'a, A, F> {
    type T = F::T;

    #[inline(always)]
    fn collide(&mut self, a: PMut<Self::T>, b: PMut<Self::T>) {
        //only check if the opoosite axis intersects.
        //already know they intersect
        let a2 = self.axis.next();
        if a.get().get_range(a2).intersects(b.get().get_range(a2)) {
            self.a.collide(a, b);
        }
    }
}

//Calls colliding on all aabbs that intersect and only one aabbs
//that intsect.
#[inline(always)]
pub fn find_2d<A: Axis, F: ColMulti>(axis: A, bots: PMut<[F::T]>, clos2: &mut F) {
    let mut b: OtherAxisCollider<A, _> = OtherAxisCollider { a: clos2, axis };
    self::find(axis, bots, &mut b);
}

//Calls colliding on all aabbs that intersect between two groups and only one aabbs
//that intsect.
#[inline(always)]
pub fn find_parallel_2d<A: Axis, F: ColMulti>(
    axis: A,
    bots1: PMut<[F::T]>,
    bots2: PMut<[F::T]>,
    clos2: &mut F,
) {
    let mut b: OtherAxisCollider<A, _> = OtherAxisCollider { a: clos2, axis };

    self::find_bijective_parallel(axis, (bots1, bots2), &mut b);
}

//Calls colliding on all aabbs that intersect between two groups and only one aabbs
//that intsect.
pub fn find_perp_2d1<A: Axis, F: ColMulti>(
    axis: A, //the axis of r1.
    r1: PMut<[F::T]>,
    r2: PMut<[F::T]>,
    clos2: &mut F,
) {
    //option1 is slightly faster than option 2.
    //but requires dynamic allocation.
    //option3 is the slowest.
    //
    //OPTION 1

    #[inline(always)]
    pub fn compare_bots<T: Aabb, K: Aabb<Num = T::Num>>(
        axis: impl Axis,
        a: &T,
        b: &K,
    ) -> core::cmp::Ordering {
        let (p1, p2) = (a.get().get_range(axis).start, b.get().get_range(axis).start);
        if p1 > p2 {
            core::cmp::Ordering::Greater
        } else {
            core::cmp::Ordering::Less
        }
    }
    let mut b: OtherAxisCollider<A, _> = OtherAxisCollider { a: clos2, axis };

    let mut rr1: Vec<PMut<F::T>> = r1.iter_mut().collect();

    rr1.sort_unstable_by(|a, b| compare_bots(axis, a, b));
    self::find_bijective_parallel2(axis, (r2, PMut::new(&mut rr1)), |a| a.flatten(), &mut b);

    //exploit the fact that they are sorted along an axis to
    //reduce the number of checks.
    //TODO check which range is smaller???
    // OPTION2
    /*
    let mut b: OtherAxisCollider<A, _> = OtherAxisCollider { a: clos2, axis };

    for y in r1.iter_mut(){
        self.find_bijective_parallel(axis,(r2.borrow_mut(),y.into_slice()),&mut b);
    }
    */

    //OPTION3
    // benched and this is the slowest.
    /*
    for mut inda in r1.iter_mut() {
        for mut indb in r2.borrow_mut().iter_mut() {
            if inda.get().intersects_rect(indb.get()) {
                clos2.collide(inda.borrow_mut(), indb.borrow_mut());
            }
        }
    }
    */
}

///Find colliding pairs using the mark and sweep algorithm.
fn find<'a, A: Axis, F: ColMulti>(axis: A, collision_botids: PMut<'a, [F::T]>, func: &mut F) {
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

    let mut active: Vec<PMut<F::T>> = Vec::new();
    //let active = self.helper.get_empty_vec_mut();

    for mut curr_bot in collision_botids.iter_mut() {
        {
            {
                let crr = curr_bot.get().get_range(axis);
                //change this to do retain and then iter
                active.retain(move |that_bot| that_bot.get().get_range(axis).end > crr.start);
            }

            for that_bot in active.iter_mut() {
                debug_assert!(curr_bot
                    .get()
                    .get_range(axis)
                    .intersects(that_bot.get().get_range(axis)));

                func.collide(curr_bot.borrow_mut(), that_bot.borrow_mut());
            }
        }
        active.push(curr_bot);
    }
}

// needed for OPTION1
fn find_bijective_parallel2<'a, A: Axis, F: ColMulti, K>(
    axis: A,
    cols: (PMut<'a, [F::T]>, PMut<[K]>),
    mut conv: impl FnMut(PMut<K>) -> PMut<F::T>,
    func: &mut F,
) {
    let mut xs = cols.0.iter_mut().peekable();
    let ys = cols.1.iter_mut();

    //let active_x = self.helper.get_empty_vec_mut();
    let mut active_x: Vec<PMut<F::T>> = Vec::new();
    for y in ys {
        let mut y = conv(y);
        //Add all the x's that are touching the y to the active x.
        for x in
            xs.peeking_take_while(|x| x.get().get_range(axis).start <= y.get().get_range(axis).end)
        {
            active_x.push(x);
        }

        //Prune all the x's that are no longer touching the y.
        active_x.retain(|x| x.get().get_range(axis).end > y.get().get_range(axis).start);

        //So at this point some of the x's could actualy not intersect y.
        //These are the x's that are to the complete right of y.
        //So to handle collisions, we want to make sure to not hit these.
        //That is why we have that condition to break out of the below loop
        for x in active_x.iter_mut() {
            if x.get().get_range(axis).start >= y.get().get_range(axis).end {
                break;
            }

            debug_assert!(x.get().get_range(axis).intersects(y.get().get_range(axis)));
            func.collide(x.borrow_mut(), y.borrow_mut());
        }
    }
}

fn find_bijective_parallel<'a, A: Axis, F: ColMulti>(
    axis: A,
    cols: (PMut<'a, [F::T]>, PMut<'a, [F::T]>),
    func: &mut F,
) {
    self::find_bijective_parallel2(axis, cols, |a| a, func)
}

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
    };
    impl ColMulti for Test {
        type T = BBox<isize, Bot>;
        fn collide(&mut self, a: PMut<BBox<isize, Bot>>, b: PMut<BBox<isize, Bot>>) {
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

    //let mut left=[b.make(0,10)];
    //let mut right=[b.make(-5,5),b.make(5,15),b.make(-5,15),b.make(2,8),b.make(-5,-6),b.make(12,13)];

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

    //let mut left=[b.make(0,10),b.make(5,20)];
    //let mut right=[b.make(16,20)];

    let mut test1 = Test {
        set: BTreeSet::new(),
    };
    self::find_bijective_parallel(
        axgeom::XAXIS,
        (PMut::new(&mut left), PMut::new(&mut right)),
        &mut test1,
    );

    let mut test2 = Test {
        set: BTreeSet::new(),
    };
    self::find_bijective_parallel(
        axgeom::XAXIS,
        (PMut::new(&mut right), PMut::new(&mut left)),
        &mut test2,
    );

    let num = test1.set.symmetric_difference(&test2.set).count();

    assert_eq!(num, 0);
}

//this can have some false positives.
//but it will still prune a lot of bots.
pub(crate) fn get_section<'a, I: Aabb, A: Axis>(
    axis: A,
    arr: &'a [I],
    range: Range<I::Num>,
) -> &'a [I] {
    if arr.is_empty() {
        return arr;
    }

    let ll = arr.len();
    let mut start = None;
    for (e, i) in arr.as_ref().iter().enumerate() {
        let rr = i.get().get_range(axis);
        if e == ll - 1 || rr.end >= range.start {
            start = Some(e);
            break;
        }
    }
    //TODO get rid of unwrap?
    let start = start.unwrap();

    let mut end = arr.as_ref().len();
    for (e, i) in arr[start..].iter().enumerate() {
        let rr = i.get().get_range(axis);
        if rr.start > range.end {
            end = start + e;
            break;
        }
    }

    //println!("{:?},{},[{},{}]",range,ll,start,end);

    &arr[start..end]
}

#[test]
fn test_section() {
    use axgeom::rect;
    let mut aabbs = [
        rect(1, 4, 0, 0),
        rect(3, 6, 0, 0),
        rect(5, 20, 0, 0),
        rect(6, 50, 0, 0),
        rect(11, 15, 0, 0),
    ];

    let k = get_section_mut(
        axgeom::XAXIS,
        PMut::new(&mut aabbs),
        axgeom::Range::new(5, 10),
    );
    let k: &[axgeom::Rect<isize>] = &k;
    assert_eq!(k.len(), 3);
}

//this can have some false positives.
//but it will still prune a lot of bots.
pub(crate) fn get_section_mut<'a, I: Aabb, A: Axis>(
    axis: A,
    mut arr: PMut<'a, [I]>,
    range: Range<I::Num>,
) -> PMut<'a, [I]> {
    if arr.is_empty() {
        return arr;
    }

    let ll = arr.len();
    let mut start = None;
    for (e, i) in arr.as_ref().iter().enumerate() {
        let rr = i.get().get_range(axis);
        if e == ll - 1 || rr.end >= range.start {
            start = Some(e);
            break;
        }
    }
    //TODO get rid of unwrap?
    let start = start.unwrap();

    let mut end = arr.as_ref().len();
    for (e, i) in arr.borrow_mut().truncate_from(start..).iter().enumerate() {
        let rr = i.get().get_range(axis);
        if rr.start > range.end {
            end = start + e;
            break;
        }
    }

    //println!("{:?},{},[{},{}]",range,ll,start,end);

    arr.truncate(start..end)
}
