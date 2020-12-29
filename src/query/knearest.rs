//! Knearest query module

use crate::query::inner_prelude::*;
use core::cmp::Ordering;

///The geometric functions that the user must provide.
pub trait Knearest {
    type T: Aabb<Num = Self::N>;
    type N: Num;

    ///User define distance function from a point to an axis aligned line of infinite length.
    fn distance_to_aaline<A: Axis>(
        &mut self,
        point: Vec2<Self::N>,
        axis: A,
        val: Self::N,
    ) -> Self::N;

    ///User defined inexpensive distance function that that can be overly conservative.
    ///It may be that the precise distance function is fast enough, in which case you can simply
    ///return None.
    fn distance_to_broad(&mut self, point: Vec2<Self::N>, a: PMut<Self::T>) -> Option<Self::N>;

    ///User defined expensive distance function. Here the user can return fine-grained distance
    ///of the shape contained in T instead of its bounding box.
    fn distance_to_fine(&mut self, point: Vec2<Self::N>, a: PMut<Self::T>) -> Self::N;
}

///Create a handler that treats each object as its aabb rectangle shape.
pub fn default_rect_knearest<T: Aabb>(
    tree: &Tree<T>,
) -> impl Knearest<T = T, N = T::Num>
where
    T::Num: num_traits::Signed + num_traits::Zero,
{
    use num_traits::Signed;
    use num_traits::Zero;
    from_closure(
        tree,
        (),
        |_, _, _| None,
        |_, point, a| {
            a.get()
                .distance_squared_to_point(point)
                .unwrap_or_else(T::Num::zero)
        },
        |_, point, a| (point.x - a).abs() * (point.x - a).abs(),
        |_, point, a| (point.y - a).abs() * (point.y - a).abs(),
    )
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

    fn distance_to_broad(&mut self, point: Vec2<Self::N>, rect: PMut<Self::T>) -> Option<Self::N> {
        self.0.distance_to_broad(point, rect)
    }

    fn distance_to_fine(&mut self, point: Vec2<Self::N>, bot: PMut<Self::T>) -> Self::N {
        self.0.distance_to_fine(point, bot)
    }
}

use crate::Tree;
///Construct an object that implements [`Knearest`] from closures.
///We pass the tree so that we can infer the type of `T`.
///
/// `fine` is a function that gives the true distance between the `point`
/// and the specified tree element.
///
/// `broad` is a function that gives the distance between the `point`
/// and the closest point of a axis aligned rectangle. This function
/// is used as a conservative estimate to prune out elements which minimizes
/// how often the `fine` function gets called.
///
/// `xline` is a function that gives the distance between the point and a axis aligned line
///    that was a fixed x value and spans the y values.
///
/// `yline` is a function that gives the distance between the point and a axis aligned line
///    that was a fixed x value and spans the y values.
///
/// `acc` is a user defined object that is passed to every call to either
/// the `fine` or `broad` functions.
///
pub fn from_closure<Acc, T: Aabb>(
    _tree: &Tree<T>,
    acc: Acc,
    broad: impl FnMut(&mut Acc, Vec2<T::Num>, PMut<T>) -> Option<T::Num>,
    fine: impl FnMut(&mut Acc, Vec2<T::Num>, PMut<T>) -> T::Num,
    xline: impl FnMut(&mut Acc, Vec2<T::Num>, T::Num) -> T::Num,
    yline: impl FnMut(&mut Acc, Vec2<T::Num>, T::Num) -> T::Num,
) -> impl Knearest<T = T, N = T::Num> {
    ///Container of closures that implements [`Knearest`]
    struct KnearestClosure<T: Aabb, Acc, B, C, D, E> {
        pub _p: PhantomData<T>,
        pub acc: Acc,
        pub broad: B,
        pub fine: C,
        pub xline: D,
        pub yline: E,
    }

    impl<'a, T: Aabb, Acc, B, C, D, E> Knearest for KnearestClosure<T, Acc, B, C, D, E>
    where
        B: FnMut(&mut Acc, Vec2<T::Num>, PMut<T>) -> Option<T::Num>,
        C: FnMut(&mut Acc, Vec2<T::Num>, PMut<T>) -> T::Num,
        D: FnMut(&mut Acc, Vec2<T::Num>, T::Num) -> T::Num,
        E: FnMut(&mut Acc, Vec2<T::Num>, T::Num) -> T::Num,
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

        fn distance_to_broad(
            &mut self,
            point: Vec2<Self::N>,
            rect: PMut<Self::T>,
        ) -> Option<Self::N> {
            (self.broad)(&mut self.acc, point, rect)
        }

        fn distance_to_fine(&mut self, point: Vec2<Self::N>, bot: PMut<Self::T>) -> Self::N {
            (self.fine)(&mut self.acc, point, bot)
        }
    }
    KnearestClosure {
        _p: PhantomData,
        acc,
        broad,
        fine,
        xline,
        yline,
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
        if let Some(long_dis) = knear.distance_to_broad(*point, curr_bot.borrow_mut()) {
            if self.curr_num == self.num {
                if let Some(l) = self.bots.last() {
                    if long_dis > l.mag {
                        return false;
                    }
                }
            }
        }
        let curr_dis = knear.distance_to_fine(*point, curr_bot.borrow_mut());

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
    #[inline(always)]
    pub fn iter(
        &mut self,
    ) -> impl Iterator<Item = &mut [KnearestResult<'a, T>]>
           + core::iter::FusedIterator
           + DoubleEndedIterator {
        crate::util::SliceSplitMut::new(&mut self.inner, |a, b| a.mag == b.mag).fuse()
    }

    ///Return the underlying datastructure
    #[inline(always)]
    pub fn into_vec(self) -> Vec<KnearestResult<'a, T>> {
        self.inner
    }

    ///returns the total number of elements counting ties
    #[inline(always)]
    pub fn total_len(&self) -> usize {
        self.inner.len()
    }
    ///Returns the number of unique distances
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.num_entires
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

use crate::container::TreeRef;
///Panics if a disconnect is detected between tree and naive queries.
pub fn assert_k_nearest_mut<T: Aabb>(
    tree: &mut TreeRef<T>,
    point: Vec2<T::Num>,
    num: usize,
    knear: &mut impl Knearest<T = T, N = T::Num>,
) {
    let bots = tree.get_bbox_elements_mut();
    use core::ops::Deref;

    fn into_ptr_usize<T>(a: &T) -> usize {
        a as *const T as usize
    }
    let mut res_naive = naive_k_nearest_mut(bots, point, num, knear)
        .into_vec()
        .drain(..)
        .map(|a| (into_ptr_usize(a.bot.deref()), a.mag))
        .collect::<Vec<_>>();

    let r = tree.k_nearest_mut(point, num, knear);
    let mut res_dino: Vec<_> = r
        .into_vec()
        .drain(..)
        .map(|a| (into_ptr_usize(a.bot.deref()), a.mag))
        .collect();

    res_naive.sort_by(|a, b| a.partial_cmp(b).unwrap());
    res_dino.sort_by(|a, b| a.partial_cmp(b).unwrap());

    assert_eq!(res_naive.len(), res_dino.len());
    assert!(res_naive.iter().eq(res_dino.iter()));
}

///Naive implementation
pub fn naive_k_nearest_mut<'a, T: Aabb>(
    elems: PMut<'a, [T]>,
    point: Vec2<T::Num>,
    num: usize,
    k: &mut impl Knearest<T = T, N = T::Num>,
) -> KResult<'a, T> {
    let mut closest = ClosestCand::new(num);

    for b in elems.iter_mut() {
        closest.consider(&point, k, b);
    }

    let num_entires = closest.curr_num;
    KResult {
        num_entires,
        inner: closest.into_sorted(),
    }
}

use super::Queries;

///Knearest functions that can be called on a tree.
pub trait KnearestQuery<'a>: Queries<'a> {
    /// Find the closest `num` elements to the specified `point`.
    /// The user provides two functions:
    ///
    /// The result is returned as one `Vec`. The closest elements will
    /// appear first. Multiple elements can be returned
    /// with the same distance in the event of ties. These groups of elements are seperated by
    /// one entry of `Option::None`. In order to iterate over each group,
    /// try using the slice function: `arr.split(|a| a.is_none())`
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
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// let mut handler = broccoli::query::knearest::from_closure(
    ///    &tree,
    ///    (),
    ///    |_, point, a| Some(a.rect.distance_squared_to_point(point).unwrap_or(0)),
    ///    |_, point, a| a.inner.distance_squared_to_point(point),
    ///    |_, point, a| distance_squared(point.x,a),
    ///    |_, point, a| distance_squared(point.y,a),
    /// );
    ///
    /// let mut res = tree.k_nearest_mut(
    ///       vec2(30, 30),
    ///       2,
    ///       &mut handler
    /// );
    ///
    /// assert_eq!(res.len(),2);
    /// assert_eq!(res.total_len(),2);
    ///
    /// let foo:Vec<_>=res.iter().map(|a|*a[0].bot.inner).collect();
    ///
    /// assert_eq!(foo,vec![vec2(7,7),vec2(5,5)]);
    ///
    ///
    /// fn distance_squared(a:isize,b:isize)->isize{
    ///     let a=(a-b).abs();
    ///     a*a
    /// }
    ///```
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
        let dt = self.vistr_mut().with_depth(Depth(0));

        let knear = KnearestBorrow(ktrait);

        let closest = ClosestCand::new(num);

        let mut blap = Blap {
            knear,
            point,
            closest,
        };

        recc(default_axis(), dt, &mut blap);

        let num_entires = blap.closest.curr_num;
        KResult {
            num_entires,
            inner: blap.closest.into_sorted(),
        }
    }
}
