//! Knearest query module

use super::*;

///The geometric functions that the user must provide.
pub trait Knearest<T: Aabb> {
    ///User define distance function from a point to an axis aligned line of infinite length.
    fn distance_1d(&mut self, start: T::Num, val: T::Num) -> T::Num;

    ///User defined inexpensive distance function that that can be overly conservative.
    ///It may be that the precise distance function is fast enough, in which case you can simply
    ///return None. If None is desired, every call to this function for a particular element must
    ///always return None.
    fn distance_to_broad(&mut self, a: &T) -> Option<T::Num>;

    ///User defined expensive distance function. Here the user can return fine-grained distance
    ///of the shape contained in T instead of its bounding box.
    fn distance_to_fine(&mut self, a: &T) -> T::Num;

    fn source(&self) -> [&T::Num; 2];
}



impl<'a, T: Aabb> Tree<'a, T> {
    pub fn find_knearest<KK: Knearest<T>>(&mut self, num: usize, ktrait: KK) -> (KK, KResult<T>) {
        let dt = self.vistr_mut().with_depth(Depth(0));

        let knear = ktrait;

        let closest = ClosestCand::new(num);

        let mut rec = Recurser { knear, closest };

        rec.recc(default_axis(), dt);

        let num_entries = rec.closest.curr_num;
        (
            rec.knear,
            KResult {
                num_entries,
                inner: rec.closest.into_sorted(),
            },
        )
    }

    pub fn find_knearest_closure<P: Point<Num = T::Num>>(
        &mut self,
        point: P,
        num: usize,
        broad: impl FnMut(&P, &T) -> Option<T::Num>,
        fine: impl FnMut(&P, &T) -> T::Num,
        dis_1d: impl FnMut(&P, T::Num, T::Num) -> T::Num,
    ) -> KResult<T> {
        let a = KnearestClosure {
            point,
            broad,
            fine,
            xline: dis_1d,
        };
        self.find_knearest(num, a).1
    }
}

macro_rules! impl_float {
    ( $x:ty ,$name:ident,$zero:expr) => {
        ///
        /// Find nearest using just axis alined bounding boxes. No fine-grained.
        ///
        pub struct $name {
            pub x: $x,
            pub y: $x,
        }

        impl<T: Aabb<Num = $x>> Knearest<T> for $name {
            fn source(&self) -> [&T::Num; 2] {
                [&self.x, &self.y]
            }
            fn distance_1d(&mut self, start: T::Num, a: T::Num) -> T::Num {
                (start - a).abs() * (start - a).abs()
            }

            fn distance_to_broad(&mut self, _rect: &T) -> Option<T::Num> {
                None
            }

            fn distance_to_fine(&mut self, aabb: &T) -> T::Num {
                
                let r=aabb.make_rect();
                r
                    .distance_squared_to_point(vec2(self.x,self.y))
                    .unwrap_or_else(||$zero)
                    
            }
        }
    };
}
impl_float!(f32, AabbKnearestF32, 0.0);
impl_float!(f64, AabbKnearestF64, 0.0);
impl_float!(isize, AabbKnearestIsize, 0);

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
///Container of closures that implements [`Knearest`]
pub struct KnearestClosure<P, B, C, D> {
    pub point: P,
    pub broad: B,
    pub fine: C,
    pub xline: D,
}

impl<P: Point<Num = T::Num>, T: Aabb, B, C, D> Knearest<T> for KnearestClosure<P, B, C, D>
where
    B: FnMut(&P, &T) -> Option<T::Num>,
    C: FnMut(&P, &T) -> T::Num,
    D: FnMut(&P, T::Num, T::Num) -> T::Num,
{
    fn distance_1d(&mut self, start: T::Num, val: T::Num) -> T::Num {
        (self.xline)(&self.point, start, val)
    }

    fn distance_to_broad(&mut self, rect: &T) -> Option<T::Num> {
        (self.broad)(&self.point, rect)
    }

    fn distance_to_fine(&mut self, bot: &T) -> T::Num {
        (self.fine)(&self.point, bot)
    }

    fn source(&self) -> [&T::Num; 2] {
        self.point.get()
    }
}

// impl<T: Aabb, K: Knearest<T>> Knearest<T> for &mut K {
//     fn distance_to_aaline<A: Axis>(&mut self, point: Vec2<T::Num>, axis: A, val: T::Num) -> T::Num {
//         (*self).distance_to_aaline(point, axis, val)
//     }

//     fn distance_to_broad(&mut self, point: Vec2<T::Num>, rect: AabbPin<&mut T>) -> Option<T::Num> {
//         (*self).distance_to_broad(point, rect)
//     }

//     fn distance_to_fine(&mut self, point: Vec2<T::Num>, bot: AabbPin<&mut T>) -> T::Num {
//         (*self).distance_to_fine(point, bot)
//     }
// }

/// Returned by k_nearest_mut
#[derive(Debug)]
pub struct KnearestResult<'a, T: Aabb> {
    pub bot: AabbPin<&'a mut T>,
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

    fn consider<K: Knearest<T>>(&mut self, knear: &mut K, curr_bot: AabbPin<&'a mut T>) {
        if let Some(long_dis) = knear.distance_to_broad(&*curr_bot) {
            if self.curr_num == self.num {
                if let Some(l) = self.bots.last() {
                    if long_dis > l.mag {
                        return;
                    }
                }
            }
        }
        let curr_dis = knear.distance_to_fine(&*curr_bot);

        let arr = &mut self.bots;

        let mut insert_index = None;

        //The closest bots are at the start.

        for (i, a) in arr.iter().enumerate() {
            if curr_dis < a.mag {
                //If we find a bot closer than everything we've had before,
                //start a new group.

                insert_index = Some(i);
                self.curr_num += 1;
                break;
            } else if curr_dis == a.mag {
                //If we find a bot at the same distance of another bot, add it to that group.
                insert_index = Some(i);
                break;
            }
        }

        if let Some(i) = insert_index {
            arr.insert(
                i,
                KnearestResult {
                    bot: curr_bot,
                    mag: curr_dis,
                },
            );

            //If we have too many groups, delete the group thats furthest away.
            if self.curr_num > self.num {
                //We know its not empty if we have gotten here
                let last_mag = arr.last().unwrap().mag;
                self.curr_num -= 1;
                while let Some(k) = arr.last() {
                    if k.mag == last_mag {
                        arr.pop();
                    } else {
                        break;
                    }
                }
            }
        } else {
            //If at this point, we havent found a place to insert the bot, check if we can just
            //make a new group at the end.
            if self.curr_num < self.num {
                self.curr_num += 1;
                arr.insert(
                    arr.len(),
                    KnearestResult {
                        bot: curr_bot,
                        mag: curr_dis,
                    },
                );
            }
        }
    }

    fn full_and_max_distance(&self) -> Option<T::Num> {
        assert!(crate::queries::is_sorted_by(&self.bots, |a, b| a
            .mag
            .partial_cmp(&b.mag)));

        if self.curr_num == self.num {
            self.bots.last().map(|a| a.mag)
        } else {
            None
        }
    }
}

struct Recurser<'a, T: Aabb, K: Knearest<T>> {
    knear: K,
    closest: ClosestCand<'a, T>,
}

impl<'a, T: Aabb, K: Knearest<T>> Recurser<'a, T, K> {
    fn should_recurse<A: Axis>(&mut self, line: (A, T::Num)) -> bool {
        if let Some(m) = self.closest.full_and_max_distance() {
            let dis = if line.0.is_xaxis() {
                self.knear.distance_1d(*self.knear.source()[0], line.1)
            } else {
                self.knear.distance_1d(*self.knear.source()[1], line.1)
            };
            dis < m
        } else {
            true
        }
    }

    fn recc<'b: 'a, A: Axis>(
        &mut self,
        axis: A,
        stuff: LevelIter<VistrMutPin<'a, Node<'b, T, T::Num>>>,
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
                if **self.knear.source().get_axis(axis) < div {
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
                    match nn.cont.contains_ext(**self.knear.source().get_axis(axis)) {
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
                self.closest.consider(&mut self.knear, bot);
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
        use slice_group_by::GroupByMut;
        self.inner.linear_group_by_mut(|a, b| a.mag == b.mag)
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

mod assert {
    use super::*;

    impl<'a, T: Aabb> Naive<'a, T> {
        pub fn find_knearest<KK: Knearest<T>>(
            &mut self,
            num: usize,
            mut ktrait: KK,
        ) -> (KK, KResult<T>) {
            let mut closest = ClosestCand::new(num);

            for b in self.inner.borrow_mut().iter_mut() {
                closest.consider(&mut ktrait, b);
            }

            let num_entries = closest.curr_num;
            (
                ktrait,
                KResult {
                    num_entries,
                    inner: closest.into_sorted(),
                },
            )
        }

        // pub fn find_knearest_closure(
        //     &mut self,
        //     point: Vec2<T::Num>,
        //     num: usize,
        //     broad: impl FnMut(Vec2<T::Num>, AabbPin<&mut T>) -> Option<T::Num>,
        //     fine: impl FnMut(Vec2<T::Num>, AabbPin<&mut T>) -> T::Num,
        //     xline: impl FnMut(Vec2<T::Num>, T::Num) -> T::Num,
        //     yline: impl FnMut(Vec2<T::Num>, T::Num) -> T::Num,
        // ) -> KResult<T> {
        //     let a = KnearestClosure {
        //         broad,
        //         fine,
        //         xline,
        //         yline,
        //     };
        //     self.find_knearest(point, num, a)
        // }
    }

    impl<'a, T: Aabb + ManySwap> Assert<'a, T> {
        ///Panics if a disconnect is detected between tree and naive queries.
        pub fn assert_k_nearest_mut<K: Knearest<T>>(&mut self, num: usize, knear: K) -> K {
            use core::ops::Deref;

            //TODO remove ptr crap
            fn into_ptr_usize<T>(a: &T) -> usize {
                a as *const T as usize
            }

            let mut tree = Tree::new(self.inner);
            let (ooo, r) = tree.find_knearest(num, knear);
            let mut res_dino: Vec<_> = r
                .into_vec()
                .drain(..)
                .map(|a| (into_ptr_usize(a.bot.deref()), a.mag))
                .collect();

            let mut nn = Naive::new(self.inner);
            let (ooo, res_naive) = nn.find_knearest(num, ooo);

            let mut res_naive = res_naive
                .into_vec()
                .drain(..)
                .map(|a| (into_ptr_usize(a.bot.deref()), a.mag))
                .collect::<Vec<_>>();

            res_naive.sort_by(|a, b| a.partial_cmp(b).unwrap());
            res_dino.sort_by(|a, b| a.partial_cmp(b).unwrap());

            assert_eq!(res_naive.len(), res_dino.len());
            assert!(res_naive.iter().eq(res_dino.iter()));
            ooo
        }
    }
}
