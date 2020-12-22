use super::super::ColMulti;
use crate::query::inner_prelude::*;
use crate::util::PreVecMut;
use super::tools::RetainMutUnordered;




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
pub fn find_2d<A: Axis, F: ColMulti>(prevec1:&mut PreVecMut<F::T>,axis: A, bots: PMut<[F::T]>, clos2: &mut F) {
    let mut b: OtherAxisCollider<A, _> = OtherAxisCollider { a: clos2, axis };
    self::find(prevec1,axis, bots, &mut b);
}

//Calls colliding on all aabbs that intersect between two groups and only one aabbs
//that intsect.
pub fn find_parallel_2d<A: Axis, F: ColMulti>(
    prevec1:&mut PreVecMut<F::T>,
    prevec2:&mut PreVecMut<F::T>, 
    axis: A,
    bots1: PMut<[F::T]>,
    bots2: PMut<[F::T]>,
    clos2: &mut F,
) {
    let mut b: OtherAxisCollider<A, _> = OtherAxisCollider { a: clos2, axis };

    self::find_other_parallel3(prevec1,prevec2,axis,(bots1,bots2),&mut b)
   
}

//Calls colliding on all aabbs that intersect between two groups and only one aabbs
//that intsect.
pub fn find_perp_2d1<A: Axis, F: ColMulti>(
    prevec1:&mut PreVecMut<F::T>, 
    prevec2:&mut PreVecMut<F::T>,
    axis: A, //the axis of r1.
    mut r1: PMut<[F::T]>,
    mut r2: PMut<[F::T]>,
    clos2: &mut F,
) {
    //option1 is slightly faster than option 2.
    //but requires dynamic allocation.
    //option3 is the slowest.
    //
    //OPTION 1
    /*
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

    let mut rr1=prevec3.get_empty_vec_mut();
    rr1.extend(r1);
    //let mut rr1: Vec<PMut<F::T>> = r1.iter_mut().collect();

    rr1.sort_unstable_by(|a, b| compare_bots(axis, a, b));

    let rrr:&mut [PMut<F::T>]=(&mut rr1) as &mut [PMut<F::T>];
    self::find_other_parallel3(
        prevec1,
        prevec2,
        axis,
        (
            r2,
            rrr.iter_mut().map(|a|PMut::new(a).flatten())
        ),
        &mut b);
    */
    //exploit the fact that they are sorted along an axis to
    //reduce the number of checks.
    // OPTION2
    
    let mut b: OtherAxisCollider<A, _> = OtherAxisCollider { a: clos2, axis };

    for y in r1.iter_mut(){
        self::find_other_parallel3(prevec1,prevec2,axis,(r2.borrow_mut(),y.into_slice()),&mut b);
    }
    

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

#[inline(always)]
///Find colliding pairs using the mark and sweep algorithm.
fn find<'a, A: Axis, F: ColMulti>(prevec1:&mut PreVecMut<F::T>,axis: A, collision_botids: PMut<'a, [F::T]>, func: &mut F) {
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

    let active=prevec1.get_empty_vec_mut();
    //let mut active: Vec<PMut<F::T>> = Vec::new();

    for mut curr_bot in collision_botids.iter_mut() {
        let crr = *curr_bot.get().get_range(axis);

        active.retain_mut_unordered(|that_bot| {
            if that_bot.get().get_range(axis).end > crr.start {
                debug_assert!(curr_bot
                    .get()
                    .get_range(axis)
                    .intersects(that_bot.get().get_range(axis)));

                func.collide(curr_bot.borrow_mut(), that_bot.borrow_mut());
                true
            } else {
                false
            }
        });

        active.push(curr_bot);
    }
}


#[inline(always)]
//does less comparisons than option 2.
fn find_other_parallel3<'a, 'b,A: Axis, F: ColMulti>(
    prevec1:&mut PreVecMut<F::T>,
    prevec2:&mut PreVecMut<F::T>, 
    axis: A,
    cols: (impl IntoIterator<Item=PMut<'a,F::T>>,impl IntoIterator<Item=PMut<'b,F::T>>),
    func: &mut F,
) where F::T:'a+'b{
    let mut f1=cols.0.into_iter().peekable();
    let mut f2=cols.1.into_iter().peekable();

    //let mut active_x: Vec<PMut<F::T>> = Vec::new();
    //let mut active_y: Vec<PMut<F::T>> = Vec::new();
    let active_x=prevec1.get_empty_vec_mut();
    let active_y=prevec2.get_empty_vec_mut();
    loop{
        let j=match (f1.peek(),f2.peek()){
            (Some(x),None)=>{
                (Some(x),None)
            },
            (None,Some(x))=>{
                (None,Some(x))
            },
            (None,None)=>{
                break;
            },
            (Some(x),Some(y))=>{
                if x.get().get_range(axis).start<y.get().get_range(axis).start{
                    (Some(x),None)
                }else{
                    (None,Some(x))
                }
            }
        };
        match j{
            (Some(_),None)=>{
                let mut x=f1.next().unwrap();
                active_y.retain_mut_unordered(|y| {
                    if y.get().get_range(axis).end > x.get().get_range(axis).start {
                        func.collide(x.borrow_mut(), y.borrow_mut());
                        true
                    } else {
                        false
                    }
                });

                active_x.push(x);
            },
            (None,Some(_))=>{
                let mut y=f2.next().unwrap();
                active_x.retain_mut_unordered(|x| {
                    if x.get().get_range(axis).end > y.get().get_range(axis).start {
                        func.collide(x.borrow_mut(), y.borrow_mut());
                        true
                    } else {
                        false
                    }
                });

                active_y.push(y); 
            },
            _=>unreachable!()
        }
    }
}

//This only uses one stack, but it ends up being more comparisons.
#[allow(dead_code)]
#[inline(always)]
fn find_other_parallel2<'a, 'b,A: Axis, F: ColMulti>(
    axis: A,
    cols: (impl IntoIterator<Item=PMut<'a,F::T>>,impl IntoIterator<Item=PMut<'b,F::T>>),
    func: &mut F,
) where F::T:'a+'b{
    
    let mut xs = cols.0.into_iter().peekable();
    let ys = cols.1.into_iter();

    //let active_x = self.helper.get_empty_vec_mut();
    let mut active_x: Vec<PMut<F::T>> = Vec::new();
    for mut y in ys {

        let yr = *y.get().get_range(axis);

        //Add all the x's that are touching the y to the active x.
        for x in xs.peeking_take_while(|x| x.get().get_range(axis).start <= yr.end) {
            active_x.push(x);
        }

        //Prune all the x's that are no longer touching the y.
        active_x.retain_mut_unordered(|x| {
            if x.get().get_range(axis).end > yr.start {
                //So at this point some of the x's could actualy not intersect y.
                //These are the x's that are to the complete right of y.
                //So to handle collisions, we want to make sure to not hit these.
                //That is why have an if condition.
                if x.get().get_range(axis).start < yr.end {
                    debug_assert!(x.get().get_range(axis).intersects(&yr));
                    func.collide(x.borrow_mut(), y.borrow_mut());
                }
                true
            } else {
                false
            }
        });
    }
    
    //dbg!(active_x.capacity());
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

    analyze::sweeper_update(axgeom::XAXIS,&mut left);
    analyze::sweeper_update(axgeom::XAXIS,&mut right);

    //let mut left=[b.make(0,10),b.make(5,20)];
    //let mut right=[b.make(16,20)];

    let mut p1=PreVecMut::new();
    let mut p2=PreVecMut::new();
    let mut test1 = Test {
        set: BTreeSet::new(),
    };

    let j1:PMut<[BBox<_,_>]>=PMut::new(&mut left);
    let j2:PMut<[BBox<_,_>]>=PMut::new(&mut right);
    
    self::find_other_parallel3(
        &mut p1,
        &mut p2,
        axgeom::XAXIS,
        (j1, j2),
        &mut test1,
    );

    let mut test2 = Test {
        set: BTreeSet::new(),
    };
    let j1:PMut<[BBox<_,_>]>=PMut::new(&mut right);
    let j2:PMut<[BBox<_,_>]>=PMut::new(&mut left);
    
    self::find_other_parallel3(
        &mut p1,
        &mut p2,
        axgeom::XAXIS,
        (j1, j2),
        &mut test2,
    );

    let diff = test1.set.symmetric_difference(&test2.set);
    let num=diff.clone().count();
    let diff2:Vec<_>=diff.collect();
    //TODO this fails???
    assert_eq!(num, 0,"{:?}",&diff2);
}

//this can have some false positives.
//but it will still prune a lot of bots.
#[inline(always)]
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

    let start = start.unwrap();

    let mut end = arr.as_ref().len();
    for (e, i) in arr[start..].iter().enumerate() {
        let rr = i.get().get_range(axis);
        if rr.start > range.end {
            end = start + e;
            break;
        }
    }

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
#[inline(always)]
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

    let start = start.unwrap();

    let mut end = arr.as_ref().len();
    for (e, i) in arr.borrow_mut().truncate_from(start..).iter().enumerate() {
        let rr = i.get().get_range(axis);
        if rr.start > range.end {
            end = start + e;
            break;
        }
    }

    arr.truncate(start..end)
}
