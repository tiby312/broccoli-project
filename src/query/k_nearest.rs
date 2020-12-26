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

///The geometric functions that the user must provide.
pub trait Knearest {
    type T: Aabb<Num = Self::N>;
    type N: Num;

    fn distance_to_aaline<A: Axis>(
        &mut self,
        point: Vec2<Self::N>,
        axis: A,
        val: Self::N,
    ) -> Self::N;

    fn distance_to_rect(&mut self, point: Vec2<Self::N>, rect: &Rect<Self::N>) -> Self::N;

    ///User defined expensive distance function. Here the user can return fine-grained distance
    ///of the shape contained in T instead of its bounding box.
    fn distance_to_bot(&mut self, point: Vec2<Self::N>, bot: &Self::T) -> Self::N {
        self.distance_to_rect(point, bot.get())
    }
}

struct KnearestBorrow<'a, K>(&'a mut K);
impl<'a, K: Knearest> Knearest for KnearestBorrow<'a, K> {
    type T = K::T;
    type N = K::N;
    fn distance_to_aaline<A: Axis>(
        &mut self,
        point: Vec2<Self::N>,
        axis: A,
        val: Self::N,
    ) -> Self::N {
        self.0.distance_to_aaline(point, axis, val)
    }

    fn distance_to_rect(&mut self, point: Vec2<Self::N>, rect: &Rect<Self::N>) -> Self::N {
        self.0.distance_to_rect(point, rect)
    }

    ///User defined expensive distance function. Here the user can return fine-grained distance
    ///of the shape contained in T instead of its bounding box.
    fn distance_to_bot(&mut self, point: Vec2<Self::N>, bot: &Self::T) -> Self::N {
        self.0.distance_to_bot(point, bot)
    }
}

pub struct KnearestClosure<'a, T: Aabb, Acc, B, C, D, E> {
    pub _p: PhantomData<T>,
    pub acc: &'a mut Acc,
    pub broad: B,
    pub fine: C,
    pub xline: D,
    pub yline: E,
}
impl<
        'a,
        T: Aabb,
        Acc,
        B: FnMut(&mut Acc, Vec2<T::Num>, &Rect<T::Num>) -> T::Num,
        C: FnMut(&mut Acc, Vec2<T::Num>, &T) -> T::Num,
        D: FnMut(&mut Acc, Vec2<T::Num>, T::Num) -> T::Num,
        E: FnMut(&mut Acc, Vec2<T::Num>, T::Num) -> T::Num,
    > KnearestClosure<'a, T, Acc, B, C, D, E>
{
    pub fn new<AA: Axis>(
        _tree: &Tree<AA, T>,
        acc: &'a mut Acc,
        broad: B,
        fine: C,
        xline: D,
        yline: E,
    ) -> Self {
        KnearestClosure {
            _p: PhantomData,
            acc,
            broad,
            fine,
            xline,
            yline,
        }
    }
}
impl<
        'a,
        T: Aabb,
        Acc,
        B: FnMut(&mut Acc, Vec2<T::Num>, &Rect<T::Num>) -> T::Num,
        C: FnMut(&mut Acc, Vec2<T::Num>, &T) -> T::Num,
        D: FnMut(&mut Acc, Vec2<T::Num>, T::Num) -> T::Num,
        E: FnMut(&mut Acc, Vec2<T::Num>, T::Num) -> T::Num,
    > Knearest for KnearestClosure<'a, T, Acc, B, C, D, E>
{
    type T = T;
    type N = T::Num;

    fn distance_to_aaline<A: Axis>(
        &mut self,
        point: Vec2<Self::N>,
        axis: A,
        val: Self::N,
    ) -> Self::N {
        if axis.is_xaxis() {
            (self.xline)(self.acc, point, val)
        } else {
            (self.yline)(self.acc, point, val)
        }
    }

    fn distance_to_rect(&mut self, point: Vec2<Self::N>, rect: &Rect<Self::N>) -> Self::N {
        (self.broad)(self.acc, point, rect)
    }

    fn distance_to_bot(&mut self, point: Vec2<Self::N>, bot: &Self::T) -> Self::N {
        (self.fine)(self.acc, point, bot)
    }
}

/// Returned by k_nearest_mut
#[derive(Debug)]
pub struct KnearestResult<'a, T: Aabb> {
    pub bot: PMut<'a, T>,
    pub mag: T::Num,
}

struct ClosestCand<'a, T: Aabb> {
    //Can have multiple bots with the same mag. So the length could be bigger than num.
    bots: Vec<KnearestResult<'a, T>>,
    //The current number of different distances in the vec
    curr_num: usize,
    //The max number of different distances.
    num: usize,
}
impl<'a, T: Aabb> ClosestCand<'a, T> {
    //First is the closest
    fn into_sorted(self) -> Vec<KnearestResult<'a, T>> {
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

    fn consider<K: Knearest<T = T, N = T::Num>>(
        &mut self,
        point: &Vec2<K::N>,
        knear: &mut K,
        curr_bot: PMut<'a, T>,
    ) -> bool {
        let long_dis = knear.distance_to_rect(*point, curr_bot.get());

        if self.curr_num == self.num {
            if let Some(l) = self.bots.last() {
                if long_dis > l.mag {
                    return false;
                }
            }
        }

        let curr_dis = knear.distance_to_bot(*point, &curr_bot);

        if self.curr_num < self.num {
            let arr = &mut self.bots;

            for i in 0..arr.len() {
                if curr_dis < arr[i].mag {
                    let unit = KnearestResult {
                        bot: curr_bot,
                        mag: curr_dis,
                    }; //$unit_create!(curr_bot,curr_dis);
                    arr.insert(i, unit);
                    self.curr_num += 1;
                    return true;
                }
            }
            //only way we get here is if the above didnt return.
            let unit = KnearestResult {
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
                    let unit = KnearestResult {
                        bot: curr_bot,
                        mag: curr_dis,
                    }; //$unit_create!(curr_bot,curr_dis);
                    arr.insert(i, unit);

                    let max = arr
                        .iter()
                        .map(|a| a.mag)
                        .max_by(|a, b| {
                            if a > b {
                                Ordering::Greater
                            } else {
                                Ordering::Less
                            }
                        })
                        .unwrap();
                    assert!(max < v.mag);
                    return true;
                } else if curr_dis == arr[i].mag {
                    let unit = KnearestResult {
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

struct Blap<'a, K: Knearest> {
    knear: K,
    point: Vec2<K::N>,
    closest: ClosestCand<'a, K::T>,
}

impl<'a, K: Knearest> Blap<'a, K> {
    fn should_recurse<A: Axis>(&mut self, line: (A, K::N)) -> bool {
        if let Some(m) = self.closest.full_and_max_distance() {
            let dis = self.knear.distance_to_aaline(self.point, line.0, line.1);

            dis < m //TODO double check
        } else {
            true
        }
    }
}

fn recc<'a, 'b: 'a, T: Aabb, A: Axis, K: Knearest<N = T::Num, T = T>>(
    axis: A,
    stuff: LevelIter<VistrMut<'a, Node<'b, T>>>,
    blap: &mut Blap<'a, K>,
) {
    let ((_depth, nn), rest) = stuff.next();
    //let nn = nn.get_mut();
    let handle_node = match rest {
        Some([left, right]) => {
            let div = match nn.div {
                Some(b) => b,
                None => return,
            };

            let line = (axis, div);

            //recurse first. more likely closest is in a child.
            if *blap.point.get_axis(axis) < div {
                recc(axis.next(), left, blap);
                if blap.should_recurse(line) {
                    recc(axis.next(), right, blap);
                }
            } else {
                recc(axis.next(), right, blap);
                if blap.should_recurse(line) {
                    recc(axis.next(), left, blap);
                }
            }

            if let Some(range) = nn.cont {
                //Determine if we should handle this node or not.
                match range.contains_ext(*blap.point.get_axis(axis)) {
                    core::cmp::Ordering::Less => blap.should_recurse((axis, range.start)),
                    core::cmp::Ordering::Greater => blap.should_recurse((axis, range.end)),
                    core::cmp::Ordering::Equal => true,
                }
            } else {
                false
            }
        }
        None => true,
    };

    if handle_node {
        for bot in nn.into_range().iter_mut() {
            //let dis_sqr = blap.knear.distance_to_bot(blap.point, bot.as_ref());
            blap.closest.consider(&blap.point, &mut blap.knear, bot);
        }
    }
}

///Returned by knearest.
pub struct KResult<'a, T: Aabb> {
    num_entires: usize,
    inner: Vec<KnearestResult<'a, T>>,
}

impl<'a, T: Aabb> KResult<'a, T> {
    ///Iterators over each group of ties starting with the closest.
    ///All the elements in one group have the same distance.
    pub fn iter(
        &mut self,
    ) -> impl Iterator<Item = &mut [KnearestResult<'a, T>]>
           + core::iter::FusedIterator
           + DoubleEndedIterator {
        crate::util::SliceSplitMut::new(&mut self.inner, |a, b| a.mag == b.mag).fuse()
    }

    ///Return the underlying datastructure
    pub fn into_vec(self) -> Vec<KnearestResult<'a, T>> {
        self.inner
    }

    ///returns the total number of elements counting ties
    pub fn total_len(&self) -> usize {
        self.inner.len()
    }
    ///Returns the number of unique distances
    pub fn len(&self) -> usize {
        self.num_entires
    }
}

pub use self::mutable::k_nearest_mut;

pub use self::mutable::k_nearest_naive_mut;
mod mutable {
    use super::*;

    pub fn k_nearest_naive_mut<'a, K: Knearest<T = T, N = T::Num>, T: Aabb>(
        bots: PMut<'a, [T]>,
        point: Vec2<K::N>,
        num: usize,
        k: &mut K,
    ) -> KResult<'a, T> {
        let mut closest = ClosestCand::new(num);

        for b in bots.iter_mut() {
            closest.consider(&point, k, b);
        }

        let num_entires = closest.curr_num;
        KResult {
            num_entires,
            inner: closest.into_sorted(),
        }
    }

    pub fn k_nearest_mut<'a, 'b: 'a, A: Axis, T: Aabb>(
        axis: A,
        vistr: VistrMut<'a, Node<'b, T>>,
        point: Vec2<T::Num>,
        num: usize,
        knear: &mut impl Knearest<N = T::Num, T = T>,
    ) -> KResult<'a, T> {
        let dt = vistr.with_depth(Depth(0));

        let knear = KnearestBorrow(knear);

        let closest = ClosestCand::new(num);

        let mut blap = Blap {
            knear,
            point,
            closest,
        };

        recc(axis, dt, &mut blap);
        /*
        let mut res: Vec<Option<(PMut<'a,N::T>, N::Num)>> = Vec::new();
        for a in blap.closest.into_sorted().into_iter() {
            if let Some(Some(k)) = res.last() {
                if k.1 != a.mag {
                    res.push(None);
                }
            }
            res.push(Some((a.bot, a.mag)));
        }
        res
        */

        let num_entires = blap.closest.curr_num;
        KResult {
            num_entires,
            inner: blap.closest.into_sorted(),
        }
    }
}
