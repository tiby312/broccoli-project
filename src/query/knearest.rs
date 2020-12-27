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

    ///User define distance function from a point to a line.
    fn distance_to_aaline<A: Axis>(
        &mut self,
        point: Vec2<Self::N>,
        axis: A,
        val: Self::N,
    ) -> Self::N;

    ///User defined inexpensive distance function that that can be overly conservative.
    fn distance_to_broad(&mut self, point: Vec2<Self::N>, a: PMut<Self::T>) -> Self::N;

    ///User defined expensive distance function. Here the user can return fine-grained distance
    ///of the shape contained in T instead of its bounding box.
    fn didstance_to_fine(&mut self, point: Vec2<Self::N>, a: PMut<Self::T>) -> Self::N;
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

    fn distance_to_broad(&mut self, point: Vec2<Self::N>, rect: PMut<Self::T>) -> Self::N {
        self.0.distance_to_broad(point, rect)
    }

    fn didstance_to_fine(&mut self, point: Vec2<Self::N>, bot: PMut<Self::T>) -> Self::N {
        self.0.didstance_to_fine(point, bot)
    }
}


///Construct an object that implements [`Knearest`] from closures.
///We pass the tree so that we can infer the type of `T`.
pub fn from_closure<
    AA:Axis,
    T:Aabb,
    Acc,
    B,
    C,
    D,
    E>(_tree:&Tree<AA,T>,acc:Acc,broad:B,fine:C,xline:D,yline:E)->KnearestClosure<T,Acc,B,C,D,E>
    where  
    B: FnMut(&mut Acc, Vec2<T::Num>, PMut<T>) -> T::Num,
    C: FnMut(&mut Acc, Vec2<T::Num>, PMut<T>) -> T::Num,
    D: FnMut(&mut Acc, Vec2<T::Num>, T::Num) -> T::Num,
    E: FnMut(&mut Acc, Vec2<T::Num>, T::Num) -> T::Num{
        KnearestClosure{_p:PhantomData,acc,broad,fine,xline,yline}
    }

///Container of closures that implements [`Knearest`]
pub struct KnearestClosure<T: Aabb, Acc, B, C, D, E> {
    pub _p: PhantomData<T>,
    pub acc: Acc,
    pub broad: B,
    pub fine: C,
    pub xline: D,
    pub yline: E,
}


impl<
        'a,
        T: Aabb,
        Acc,
        B,
        C,
        D,
        E,
    > Knearest for KnearestClosure<T, Acc, B, C, D, E>
    where
        B: FnMut(&mut Acc, Vec2<T::Num>, PMut<T>) -> T::Num,
        C: FnMut(&mut Acc, Vec2<T::Num>, PMut<T>) -> T::Num,
        D: FnMut(&mut Acc, Vec2<T::Num>, T::Num) -> T::Num,
        E: FnMut(&mut Acc, Vec2<T::Num>, T::Num) -> T::Num
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
            (self.xline)(&mut self.acc, point, val)
        } else {
            (self.yline)(&mut self.acc, point, val)
        }
    }

    fn distance_to_broad(&mut self, point: Vec2<Self::N>, rect: PMut<Self::T>) -> Self::N {
        (self.broad)(&mut self.acc, point, rect)
    }

    fn didstance_to_fine(&mut self, point: Vec2<Self::N>, bot: PMut<Self::T>) -> Self::N {
        (self.fine)(&mut self.acc, point, bot)
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
        mut curr_bot: PMut<'a, T>,
    ) -> bool {
        let long_dis = knear.distance_to_broad(*point, curr_bot.borrow_mut());

        if self.curr_num == self.num {
            if let Some(l) = self.bots.last() {
                if long_dis > l.mag {
                    return false;
                }
            }
        }

        let curr_dis = knear.didstance_to_fine(*point, curr_bot.borrow_mut());

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
            //let dis_sqr = blap.knear.didstance_to_fine(blap.point, bot.as_ref());
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


use super::NaiveQueries;
impl<K:NaiveQueries> KnearestNaiveQuery for K{}
pub trait KnearestNaiveQuery:NaiveQueries{
    fn k_nearest_mut<'a, K: Knearest<T = Self::T, N = Self::Num>>(
        &'a mut self,
        point: Vec2<K::N>,
        num: usize,
        k: &mut K,
    ) -> KResult<'a, Self::T> {
        let bots=self.get_slice_mut();
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
    
}


use super::Queries;
impl<'a,K:Queries<'a>> KnearestQuery<'a> for K{}

pub trait KnearestQuery<'a>:Queries<'a>{
    ///TODO document
    /*
    /// Find the closest `num` elements to the specified `point`.
    /// The user provides two functions:
    ///
    /// * `fine` is a function that gives the true distance between the `point`
    /// and the specified tree element.
    ///
    /// * `broad` is a function that gives the distance between the `point`
    /// and the closest point of a axis aligned rectangle. This function
    /// is used as a conservative estimate to prune out elements which minimizes
    /// how often the `fine` function gets called.
    ///
    /// `border` is the starting axis axis aligned rectangle to use. This
    /// rectangle will be split up and used to prune candidated. All candidate elements
    /// should be within this starting rectangle.
    ///
    /// The result is returned as one `Vec`. The closest elements will
    /// appear first. Multiple elements can be returned
    /// with the same distance in the event of ties. These groups of elements are seperated by
    /// one entry of `Option::None`. In order to iterate over each group,
    /// try using the slice function: `arr.split(|a| a.is_none())`
    ///
    /// `acc` is a user defined object that is passed to every call to either
    /// the `fine` or `broad` functions.
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// use axgeom::vec2;
    ///
    /// let mut inner1=vec2(5,5);
    /// let mut inner2=vec2(3,3);
    /// let mut inner3=vec2(7,7);
    ///
    /// let mut bots = [bbox(rect(0,10,0,10),&mut inner1),
    ///               bbox(rect(2,4,2,4),&mut inner2),
    ///               bbox(rect(6,8,6,8),&mut inner3)];
    ///
    /// let border = broccoli::rect(0, 100, 0, 100);
    ///
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// let mut res = tree.k_nearest_mut(
    ///       vec2(30, 30),
    ///       2,
    ///       &mut (),
    ///       |(), a, b| b.distance_squared_to_point(a).unwrap_or(0),
    ///       |(), a, b| b.inner.distance_squared_to_point(a),
    ///       border,
    /// );
    ///
    /// assert_eq!(res.len(),2);
    /// assert_eq!(res.total_len(),2);
    ///
    /// let foo:Vec<_>=res.iter().map(|a|*a[0].bot.inner).collect();
    ///
    /// assert_eq!(foo,vec![vec2(7,7),vec2(5,5)])
    ///```
    */
    #[must_use]
    fn k_nearest_mut<'b, K: Knearest<T = Self::T, N = Self::Num>>(
        &'b mut self,
        point: Vec2<Self::Num>,
        num: usize,
        ktrait: &mut K,
    ) -> KResult<Self::T>
    where
        'a: 'b,
    {
        let axis=self.axis();
        let dt = self.vistr_mut().with_depth(Depth(0));

        let knear = KnearestBorrow(ktrait);

        let closest = ClosestCand::new(num);

        let mut blap = Blap {
            knear,
            point,
            closest,
        };

        recc(axis, dt, &mut blap);
        
        let num_entires = blap.closest.curr_num;
        KResult {
            num_entires,
            inner: blap.closest.into_sorted(),
        }
    }
}
