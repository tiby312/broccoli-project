//! Knearest query module

use super::*;

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
    ///return None. If None is desired, every call to this function for a particular element must
    ///always return None.
    fn distance_to_broad(&mut self, point: Vec2<Self::N>, a: PMut<Self::T>) -> Option<Self::N>;

    ///User defined expensive distance function. Here the user can return fine-grained distance
    ///of the shape contained in T instead of its bounding box.
    fn distance_to_fine(&mut self, point: Vec2<Self::N>, a: PMut<Self::T>) -> Self::N;
}

///Create a handler that treats each object as its aabb rectangle shape.
pub fn default_rect_knearest<T: Aabb>(tree: &Tree<T>) -> impl Knearest<T = T, N = T::Num>
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

///Hide the lifetime behind the RayCast trait
///to make things simpler
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
    ) {
        if let Some(long_dis) = knear.distance_to_broad(*point, curr_bot.borrow_mut()) {
            if self.curr_num == self.num {
                if let Some(l) = self.bots.last() {
                    if long_dis > l.mag {
                        return;
                    }
                }
            }
        }
        let curr_dis = knear.distance_to_fine(*point, curr_bot.borrow_mut());

        let arr = &mut self.bots;



        let mut insert_index=None;

        //The closest bots are at the start.

        for (i,a) in arr.iter().enumerate(){
            
            if curr_dis<a.mag{
                //If we find a bot closer than everything we've had before, 
                //start a new group.

                insert_index=Some(i);
                self.curr_num+=1;
                break;
            }else if curr_dis==a.mag{
                //If we find a bot at the same distance of another bot, add it to that group.
                insert_index=Some(i);
                break;
            }
        }


        if let Some(i)=insert_index{
            arr.insert(i,KnearestResult {
                bot: curr_bot,
                mag: curr_dis,
            });

            //If we have too many groups, delete the group thats furthest away.
            if self.curr_num>self.num{
                //We know its not empty if we have gotten here
                let last_mag=arr.last().unwrap().mag;
                self.curr_num-=1;
                while let Some(k)=arr.last(){
                    if k.mag==last_mag{
                        arr.pop();
                    }else{
                        break;
                    }
                }
            }
        }else{
            //If at this point, we havent found a place to insert the bot, check if we can just
            //make a new group at the end.
            if self.curr_num<self.num{
                self.curr_num+=1;
                arr.insert(arr.len(),KnearestResult {
                    bot: curr_bot,
                    mag: curr_dis,
                });
            }
        }
    }

    fn full_and_max_distance(&self) -> Option<T::Num> {
        assert!(crate::util::is_sorted_by(&self.bots, |a, b| a
            .mag
            .partial_cmp(&b.mag)));

        if self.curr_num == self.num {
            self.bots.last().map(|a| a.mag)
        } else {
            None
        }
    }
}

struct Recurser<'a, K: Knearest> {
    knear: K,
    point: Vec2<K::N>,
    closest: ClosestCand<'a, K::T>,
}

impl<'a, K: Knearest> Recurser<'a, K> {
    fn should_recurse<A: Axis>(&mut self, line: (A, K::N)) -> bool {
        if let Some(m) = self.closest.full_and_max_distance() {
            let dis = self.knear.distance_to_aaline(self.point, line.0, line.1);
            dis < m
        } else {
            true
        }
    }

    fn recc<'b: 'a, A: Axis>(&mut self, axis: A, stuff: LevelIter<VistrMut<'a, Node<'b, K::T>>>) {
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
                if *self.point.get_axis(axis) < div {
                    self.recc(axis.next(), left);
                    if self.should_recurse(line) {
                        self.recc(axis.next(), right);
                    }
                } else {
                    self.recc(axis.next(), right);
                    if self.should_recurse(line) {
                        self.recc(axis.next(), left);
                    }
                }

                if !nn.range.is_empty() {
                    //Determine if we should handle this node or not.
                    match nn.cont.contains_ext(*self.point.get_axis(axis)) {
                        core::cmp::Ordering::Less => self.should_recurse((axis, nn.cont.start)),
                        core::cmp::Ordering::Greater => self.should_recurse((axis, nn.cont.end)),
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
                self.closest.consider(&self.point, &mut self.knear, bot);
            }
        }
    }
}

///Returned by knearest.
pub struct KResult<'a, T: Aabb> {
    num_entries: usize,
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

        //use slice_group_by::GroupByMut;
        //self.inner.linear_group_by_mut(|a,b|a.mag==b.mag)
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
        self.num_entries
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

///Panics if a disconnect is detected between tree and naive queries.
pub fn assert_k_nearest_mut<T: Aabb>(
    tree: &mut Tree<T>,
    point: Vec2<T::Num>,
    num: usize,
    knear: &mut impl Knearest<T = T, N = T::Num>,
) {
    let bots = tree.get_elements_mut();
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

    let num_entries = closest.curr_num;
    KResult {
        num_entries,
        inner: closest.into_sorted(),
    }
}

pub fn knearest_mut<'a, K: Knearest>(
    tree: &'a mut Tree<K::T>,
    point: Vec2<K::N>,
    num: usize,
    ktrait: &mut K,
) -> KResult<'a, K::T> {
    let dt = tree.vistr_mut().with_depth(Depth(0));

    let knear = KnearestBorrow(ktrait);

    let closest = ClosestCand::new(num);

    let mut rec = Recurser {
        knear,
        point,
        closest,
    };

    rec.recc(default_axis(), dt);

    let num_entries = rec.closest.curr_num;
    KResult {
        num_entries,
        inner: rec.closest.into_sorted(),
    }
}
