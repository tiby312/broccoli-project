//!
//! # User Guide
//!
//! There are four flavors of the same fundamental knearest api provided in this module.
//! There is a naive version, and there is a version that uses the tree, and there are mutable versions of those
//! that return mutable references.
//!
//! Along with a reference to the tree, the user provides the needed geometric functions by passing an implementation of Knearest.
//! The user provides a point, and the number of nearest objects to return.
//! Then the equivalent to a Vec<(&mut T,N)> is returned where T is the element, and N is its distance.
//! Even if you are only looking for one closest element, becaise of ties, it is possible for many many bots to be returned.
//! If we only returned an arbitrary one, then that would make verification against the naive algorithm harder.
//! It is possible for the Vec returned to be empty if the tree does not contain any bots.
//! While ties are possible, the ordering in which the ties are returned is arbitrary and has no meaning.
//!
//! If the user looks for multiple nearest, then the Vec will return first, all the 1st closest ties,
//! then all the 2nd closest ties, etc.
//!
//! Slice splitting functions are provided that will split up the Vec into slices over just the ties.
//!

use crate::query::inner_prelude::*;
use core::cmp::Ordering;

pub struct KnearestClosure<'a, Acc, B, F, T: Aabb> {
    pub acc: &'a mut Acc,
    pub broad: B,
    pub fine: F,
    pub _p: PhantomData<T>,
}
impl<
        Acc,
        B: FnMut(&mut Acc, Vec2<T::Num>, &Rect<T::Num>) -> T::Num,
        F: FnMut(&mut Acc, Vec2<T::Num>, &T) -> T::Num,
        T: Aabb,
    > Knearest for KnearestClosure<'_, Acc, B, F, T>
{
    type T = T;
    type N = T::Num;
    fn distance_to_rect(&mut self, point: Vec2<Self::N>, rect: &Rect<Self::N>) -> Self::N {
        (self.broad)(self.acc, point, rect)
    }

    fn distance_to_bot(&mut self, point: Vec2<Self::N>, bot: &Self::T) -> Self::N {
        (self.fine)(self.acc, point, bot)
    }
}

///The geometric functions that the user must provide.
pub trait Knearest {
    type T: Aabb<Num = Self::N>;
    type N: Num;

    fn distance_to_rect(&mut self, point: Vec2<Self::N>, rect: &Rect<Self::N>) -> Self::N;

    ///User defined expensive distance function. Here the user can return fine-grained distance
    ///of the shape contained in T instead of its bounding box.
    fn distance_to_bot(&mut self, point: Vec2<Self::N>, bot: &Self::T) -> Self::N {
        self.distance_to_rect(point, bot.get())
    }
}

/*
pub(crate) struct KnearestWrapper<T: Aabb, K> {
    pub(crate) inner: K,
    pub(crate) _p: PhantomData<T>,
}
impl<T: Aabb, K: FnMut(Vec2<T::Num>, &Rect<T::Num>) -> T::Num> Knearest for KnearestWrapper<T, K> {
    type T = T;
    type N = T::Num;

    fn distance_to_rect(&mut self, point: Vec2<Self::N>, rect: &Rect<Self::N>) -> Self::N {
        (self.inner)(point, rect)
    }
}
*/

fn make_rect_from_range<A: Axis, N: Num>(axis: A, range: &Range<N>, rect: &Rect<N>) -> Rect<N> {
    if axis.is_xaxis() {
        Rect {
            x: *range,
            y: rect.y,
        }
    } else {
        Rect {
            x: rect.x,
            y: *range,
        }
    }
}

fn range_side<N: Num>(point: Vec2<N>, axis: impl axgeom::Axis, range: &Range<N>) -> Ordering {
    let v = if axis.is_xaxis() { point.x } else { point.y };
    range.contains_ext(v)
}

/// Returned by k_nearest_mut
struct InnerKnearestResult<'a, T: Aabb> {
    pub bot: PMut<'a, T>,
    pub mag: T::Num,
}

struct ClosestCand<'a, T: Aabb> {
    //Can have multiple bots with the same mag. So the length could be bigger than num.
    bots: Vec<InnerKnearestResult<'a, T>>,
    //The current number of different distances in the vec
    curr_num: usize,
    //The max number of different distances.
    num: usize,
}
impl<'a, T: Aabb> ClosestCand<'a, T> {
    //First is the closest
    fn into_sorted(self) -> Vec<InnerKnearestResult<'a, T>> {
        self.bots
    }
    fn new(num: usize) -> ClosestCand<'a, T> {
        let bots = Vec::with_capacity(num);
        ClosestCand {
            bots,
            num,
            curr_num: 0,
        }
    }

    fn consider<K:Knearest<T=T,N=T::Num>>(&mut self, point:&Vec2<K::N>,knear:&mut K,curr_bot: PMut<'a, T>) -> bool{
        
        /* TODO figure this out.
        let long_dis=knear.distance_to_rect(*point,curr_bot.get());
        
        if self.curr_num==self.num{
            if let Some(l)=self.bots.last(){
                if long_dis>l.mag{
                    return false;
                }
            }
        }
        */
        

        let curr_dis = knear.distance_to_bot(*point, curr_bot.as_ref());
                            


        if self.curr_num < self.num {
            let arr = &mut self.bots;

            for i in 0..arr.len() {
                if curr_dis < arr[i].mag {
                    let unit = InnerKnearestResult {
                        bot: curr_bot,
                        mag: curr_dis,
                    }; //$unit_create!(curr_bot,curr_dis);
                    arr.insert(i, unit);
                    self.curr_num += 1;
                    return true;
                }
            }
            //only way we get here is if the above didnt return.
            let unit = InnerKnearestResult {
                bot: curr_bot,
                mag: curr_dis,
            };
            self.curr_num += 1;
            arr.push(unit);
        } else {
            let arr = &mut self.bots;
            for i in 0..arr.len() {
                if curr_dis < arr[i].mag {
                    let v = arr.pop().unwrap();
                    loop {
                        if arr[arr.len() - 1].mag == v.mag {
                            arr.pop().unwrap();
                        } else {
                            break;
                        }
                    }
                    let unit = InnerKnearestResult {
                        bot: curr_bot,
                        mag: curr_dis,
                    }; //$unit_create!(curr_bot,curr_dis);
                    arr.insert(i, unit);

                    let max = arr.iter().map(|a| a.mag).max().unwrap();
                    assert!(max < v.mag);
                    return true;
                } else if curr_dis == arr[i].mag {
                    let unit = InnerKnearestResult {
                        bot: curr_bot,
                        mag: curr_dis,
                    }; //$unit_create!(curr_bot,curr_dis);
                    arr.insert(i, unit);
                    return true;
                }
            }
        }

        false
    }

    fn full_and_max_distance(&self) -> Option<T::Num> {
        use is_sorted::IsSorted;
        assert!(IsSorted::is_sorted(&mut self.bots.iter().map(|a| a.mag)));
        if self.curr_num == self.num {
            self.bots.last().map(|a| a.mag)
        } else {
            None
        }
    }
}

struct Blap<'a: 'b, 'b, K: Knearest> {
    knear: &'b mut K,
    point: Vec2<K::N>,
    closest: ClosestCand<'a, K::T>,
}

impl<'a: 'b, 'b, K: Knearest> Blap<'a, 'b, K> {
    fn should_traverse_rect(&mut self, rect: &Rect<K::N>) -> bool {
        if let Some(dis) = self.closest.full_and_max_distance() {
            self.knear.distance_to_rect(self.point, rect) < dis
        } else {
            true
        }
    }
}

fn recc<'a: 'b, 'b, N: Node, A: Axis, K: Knearest<N = N::Num, T = N::T>>(
    axis: A,
    stuff: LevelIter<VistrMut<'a, N>>,
    rect: Rect<K::N>,
    blap: &mut Blap<'a, 'b, K>,
) {
    let ((_depth, nn), rest) = stuff.next();
    let nn = nn.get_mut();
    match rest {
        Some([left, right]) => {
            let div = match nn.div {
                Some(b) => b,
                None => return,
            };

            let (rleft, rright) = rect.subdivide(axis, *div);

            let range = &match nn.cont {
                Some(cont) => *cont,
                None => Range {
                    start: *div,
                    end: *div,
                },
            };

            let rmiddle = make_rect_from_range(axis, range, &rect);

            match range_side(blap.point, axis, range) {
                Ordering::Less => {
                    if blap.should_traverse_rect(&rleft) {
                        recc(axis.next(), left, rleft, blap);
                    }

                    if blap.should_traverse_rect(&rmiddle) {
                        for bot in nn.bots.iter_mut() {
                            //let dis_sqr = blap.knear.distance_to_bot(blap.point, bot.as_ref());
                            blap.closest.consider(&blap.point,blap.knear,bot);
                        }
                    }

                    if blap.should_traverse_rect(&rright) {
                        recc(axis.next(), right, rright, blap);
                    }
                }
                Ordering::Greater => {
                    if blap.should_traverse_rect(&rright) {
                        recc(axis.next(), right, rright, blap);
                    }

                    if blap.should_traverse_rect(&rmiddle) {
                        for bot in nn.bots.iter_mut() {
                            //let dis_sqr = blap.knear.distance_to_bot(blap.point, bot.as_ref());
                            blap.closest.consider(&blap.point,blap.knear,bot);
                        }
                    }
                    if blap.should_traverse_rect(&rleft) {
                        recc(axis.next(), left, rleft, blap);
                    }
                }
                Ordering::Equal => {
                    //Assume there are more elements in the children than the current node,
                    //so recurse first.
                    if blap.should_traverse_rect(&rright) {
                        recc(axis.next(), right, rright, blap);
                    }
                    if blap.should_traverse_rect(&rleft) {
                        recc(axis.next(), left, rleft, blap);
                    }
                    if blap.should_traverse_rect(&rmiddle) {
                        for bot in nn.bots.iter_mut() {
                            //let dis_sqr = blap.knear.distance_to_bot(blap.point, bot.as_ref());
                            blap.closest.consider(&blap.point,blap.knear,bot);
                        }
                    }
                }
            }
        }
        None => {
            for bot in nn.bots.iter_mut() {
                //let dis_sqr = blap.knear.distance_to_bot(blap.point, bot.as_ref());
                blap.closest.consider(&blap.point,blap.knear,bot);
            }
        }
    }
}

pub use self::mutable::k_nearest_mut;

pub use self::mutable::k_nearest_naive_mut;
mod mutable {
    use super::*;

    pub fn k_nearest_naive_mut<'a, K: Knearest<T = T, N = T::Num>, T: Aabb + HasInner>(
        bots: PMut<'a, [T]>,
        point: Vec2<K::N>,
        num: usize,
        k: &mut K,
    ) -> Vec<(&'a mut T::Inner, T::Num)> {
        
        let mut closest = ClosestCand::new(num);

        for b in bots.iter_mut() {
            closest.consider(&point,k,b);
        }

        closest
            .into_sorted()
            .drain(..)
            .map(|a| (a.bot.into_inner(), a.mag))
            .collect()
    }

    pub fn k_nearest_mut<'a, A: Axis, N: Node>(
        axis: A,
        vistr: VistrMut<'a, N>,
        point: Vec2<N::Num>,
        num: usize,
        knear: &mut impl Knearest<N = N::Num, T = N::T>,
        rect: Rect<N::Num>,
    ) -> Vec<Option<(&'a mut <N::T as HasInner>::Inner, N::Num)>>
    where
        N::T: HasInner,
    {
        let dt = vistr.with_depth(Depth(0));

        let closest = ClosestCand::new(num);

        let mut blap = Blap {
            knear,
            point,
            closest,
        };

        recc(axis, dt, rect, &mut blap);

        let mut res:Vec<Option<(&'a mut <N::T as HasInner>::Inner, N::Num)>>=Vec::new();
        for a in blap.closest.into_sorted().into_iter(){
            if let Some(Some(k))=res.last(){
                if k.1!=a.mag{
                    res.push(None);
                }
            }
            res.push(Some((a.bot.into_inner(),a.mag)));
        }
        res
    }
}
