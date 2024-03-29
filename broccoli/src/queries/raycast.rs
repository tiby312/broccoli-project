//! Raycast query module

use super::*;
use axgeom::Ray;

///A `Vec<T>` is returned since there could be ties where the ray hits multiple T at a length N away.
// `pub type RayCastResult<T, N> = axgeom::CastResult<(Vec<T>, N)>;`

///This is the trait that defines raycast specific geometric functions that are needed by this raytracing algorithm.
///By containing all these functions in this trait, we can keep the trait bounds of the underlying Num to a minimum
///of only needing Ord.
pub trait RayCast<T: Aabb> {
    ///Return the cast result to a axis aligned line of infinite length.
    fn cast_to_aaline<A: Axis>(
        &mut self,
        ray: &Ray<T::Num>,
        line: A,
        val: T::Num,
    ) -> axgeom::CastResult<T::Num>;

    ///Return the cast result that is cheap and overly conservative.
    ///It may be that the precise cast is fast enough, in which case you can simply
    ///return None. If None is desired, every call to this function for a particular element must
    ///always return None.
    fn cast_broad(
        &mut self,
        ray: &Ray<T::Num>,
        a: AabbPin<&mut T>,
    ) -> Option<axgeom::CastResult<T::Num>>;

    ///Return the exact cast result.
    fn cast_fine(&mut self, ray: &Ray<T::Num>, a: AabbPin<&mut T>) -> axgeom::CastResult<T::Num>;
}

///
/// No fine-grained just cast to aabb
///
pub struct AabbRaycast;

impl<T: Aabb> RayCast<T> for AabbRaycast
where
    T::Num: core::fmt::Debug + num_traits::Signed,
{
    fn cast_to_aaline<A: Axis>(
        &mut self,
        ray: &Ray<T::Num>,
        line: A,
        val: T::Num,
    ) -> axgeom::CastResult<T::Num> {
        ray.cast_to_aaline(line, val)
    }

    fn cast_broad(
        &mut self,
        _ray: &Ray<T::Num>,
        _a: AabbPin<&mut T>,
    ) -> Option<axgeom::CastResult<T::Num>> {
        None
    }

    fn cast_fine(&mut self, ray: &Ray<T::Num>, a: AabbPin<&mut T>) -> axgeom::CastResult<T::Num> {
        ray.cast_to_rect(a.get())
    }
}

///Construct an object that implements [`RayCast`] from closures.
///We pass the tree so that we can infer the type of `T`.
///
/// `fine` is a function that returns the true length of a ray
/// cast to an object.
///
/// `broad` is a function that returns the length of a ray cast to
/// a axis aligned rectangle. This function
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
struct RayCastClosure<B, C, D, E> {
    broad: B,
    fine: C,
    xline: D,
    yline: E,
}

impl<T: Aabb, B, C, D, E> RayCast<T> for RayCastClosure<B, C, D, E>
where
    B: FnMut(&Ray<T::Num>, AabbPin<&mut T>) -> Option<CastResult<T::Num>>,
    C: FnMut(&Ray<T::Num>, AabbPin<&mut T>) -> CastResult<T::Num>,
    D: FnMut(&Ray<T::Num>, T::Num) -> CastResult<T::Num>,
    E: FnMut(&Ray<T::Num>, T::Num) -> CastResult<T::Num>,
{
    fn cast_to_aaline<X: Axis>(
        &mut self,
        ray: &Ray<T::Num>,
        line: X,
        val: T::Num,
    ) -> axgeom::CastResult<T::Num> {
        if line.is_xaxis() {
            (self.xline)(ray, val)
        } else {
            (self.yline)(ray, val)
        }
    }
    fn cast_broad(&mut self, ray: &Ray<T::Num>, a: AabbPin<&mut T>) -> Option<CastResult<T::Num>> {
        (self.broad)(ray, a)
    }

    fn cast_fine(&mut self, ray: &Ray<T::Num>, a: AabbPin<&mut T>) -> CastResult<T::Num> {
        (self.fine)(ray, a)
    }
}

impl<'a, T: Aabb> Tree<'a, T> {
    pub fn cast_ray_closure(
        &mut self,
        ray: Ray<T::Num>,
        broad: impl FnMut(&Ray<T::Num>, AabbPin<&mut T>) -> Option<CastResult<T::Num>>,
        fine: impl FnMut(&Ray<T::Num>, AabbPin<&mut T>) -> CastResult<T::Num>,
        xline: impl FnMut(&Ray<T::Num>, T::Num) -> CastResult<T::Num>,
        yline: impl FnMut(&Ray<T::Num>, T::Num) -> CastResult<T::Num>,
    ) -> axgeom::CastResult<CastAnswer<T>> {
        let d = RayCastClosure {
            broad,
            fine,
            xline,
            yline,
        };
        self.cast_ray(ray, d)
    }

    pub fn cast_ray<R: RayCast<T>>(
        &mut self,
        ray: Ray<T::Num>,
        mut rtrait: R,
    ) -> axgeom::CastResult<CastAnswer<T>> {
        struct Recurser<'a, T: Aabb, R: RayCast<T>> {
            rtrait: R,
            ray: Ray<T::Num>,
            closest: Closest<'a, T>,
        }

        impl<'a, T: Aabb, R: RayCast<T>> Recurser<'a, T, R> {
            fn should_recurse<A: Axis>(&mut self, line: (A, T::Num)) -> bool {
                match self.rtrait.cast_to_aaline(&self.ray, line.0, line.1) {
                    axgeom::CastResult::Hit(val) => match self.closest.get_dis() {
                        Some(dis) => val <= dis,
                        None => true,
                    },
                    axgeom::CastResult::NoHit => false,
                }
            }

            //Returns the first object that touches the ray.
            fn recc<'b: 'a, A: Axis>(
                &mut self,
                axis: A,
                stuff: LevelIter<VistrMutPin<'a, Node<'b, T, T::Num>>>,
            ) {
                let ((_depth, nn), rest) = stuff.next();
                let handle_curr = if let Some([left, right]) = rest {
                    let axis_next = axis.next();

                    let div = match nn.div {
                        Some(b) => b,
                        None => return,
                    };

                    let line = (axis, div);

                    //more likely to find closest in child than current node.
                    //so recurse first before handling this node.
                    if *self.ray.point.get_axis(axis) < div {
                        self.recc(axis_next, left);

                        if self.should_recurse(line) {
                            self.recc(axis_next, right);
                        }
                    } else {
                        self.recc(axis_next, right);

                        if self.should_recurse(line) {
                            self.recc(axis_next, left);
                        }
                    }

                    if !nn.range.is_empty() {
                        //Determine if we should handle this node or not.
                        match nn.cont.contains_ext(*self.ray.point.get_axis(axis)) {
                            core::cmp::Ordering::Less => self.should_recurse((axis, nn.cont.start)),
                            core::cmp::Ordering::Greater => {
                                self.should_recurse((axis, nn.cont.end))
                            }
                            core::cmp::Ordering::Equal => true,
                        }
                    } else {
                        false
                    }
                } else {
                    true
                };
                if handle_curr {
                    for b in nn.into_range().iter_mut() {
                        self.closest.consider(&self.ray, b, &mut self.rtrait);
                    }
                }
            }
        }

        let rtrait = &mut rtrait;
        let dt = self.vistr_mut().with_depth(Depth(0));

        let closest = Closest { closest: None };
        let mut rec = Recurser {
            rtrait,
            ray,
            closest,
        };
        rec.recc(default_axis(), dt);

        match rec.closest.closest {
            Some((a, b)) => axgeom::CastResult::Hit(CastAnswer { elems: a, mag: b }),
            None => axgeom::CastResult::NoHit,
        }
    }
}

impl<T: Aabb, R: RayCast<T>> RayCast<T> for &mut R {
    fn cast_to_aaline<A: Axis>(
        &mut self,
        ray: &Ray<T::Num>,
        line: A,
        val: T::Num,
    ) -> axgeom::CastResult<T::Num> {
        (*self).cast_to_aaline(ray, line, val)
    }

    fn cast_broad(
        &mut self,
        ray: &Ray<T::Num>,
        a: AabbPin<&mut T>,
    ) -> Option<axgeom::CastResult<T::Num>> {
        (*self).cast_broad(ray, a)
    }

    fn cast_fine(&mut self, ray: &Ray<T::Num>, a: AabbPin<&mut T>) -> axgeom::CastResult<T::Num> {
        (*self).cast_fine(ray, a)
    }
}

struct Closest<'a, T: Aabb> {
    closest: Option<(Vec<AabbPin<&'a mut T>>, T::Num)>,
}
impl<'a, T: Aabb> Closest<'a, T> {
    fn consider<R: RayCast<T>>(
        &mut self,
        ray: &Ray<T::Num>,
        mut b: AabbPin<&'a mut T>,
        raytrait: &mut R,
    ) {
        //first check if bounding box could possibly be a candidate.
        if let Some(broad) = raytrait.cast_broad(ray, b.borrow_mut()) {
            let y = match broad {
                axgeom::CastResult::Hit(val) => val,
                axgeom::CastResult::NoHit => {
                    return;
                }
            };

            if let Some(dis) = self.closest.as_mut() {
                if y > dis.1 {
                    //no way this bot will be a candidate, return.
                    return;
                } else {
                    //this aabb could be a candidate, continue.
                }
            } else {
                //this aabb could be a candidate, continue,
            }
        }

        let x = match raytrait.cast_fine(ray, b.borrow_mut()) {
            axgeom::CastResult::Hit(val) => val,
            axgeom::CastResult::NoHit => {
                return;
            }
        };

        match self.closest.as_mut() {
            Some(mut dis) => {
                if x > dis.1 {
                    //do nothing
                } else if x < dis.1 {
                    dis.0.clear();
                    dis.0.push(b);
                    dis.1 = x;
                } else {
                    dis.0.push(b);
                }
            }
            None => self.closest = Some((vec![b], x)),
        };
    }

    fn get_dis(&self) -> Option<T::Num> {
        self.closest.as_ref().map(|x| x.1)
    }
}

mod assert {
    use super::*;
    impl<'a, T: Aabb> Naive<'a, T> {
        pub fn cast_ray_closure(
            &mut self,
            ray: Ray<T::Num>,
            broad: impl FnMut(&Ray<T::Num>, AabbPin<&mut T>) -> Option<CastResult<T::Num>>,
            fine: impl FnMut(&Ray<T::Num>, AabbPin<&mut T>) -> CastResult<T::Num>,
            xline: impl FnMut(&Ray<T::Num>, T::Num) -> CastResult<T::Num>,
            yline: impl FnMut(&Ray<T::Num>, T::Num) -> CastResult<T::Num>,
        ) -> axgeom::CastResult<CastAnswer<T>> {
            let d = RayCastClosure {
                broad,
                fine,
                xline,
                yline,
            };
            self.cast_ray(ray, d)
        }

        pub fn cast_ray<R: RayCast<T>>(
            &mut self,
            ray: Ray<T::Num>,
            mut ar: R,
        ) -> axgeom::CastResult<CastAnswer<T>> {
            let mut closest = Closest { closest: None };

            for b in self.iter_mut() {
                closest.consider(&ray, b, &mut ar);
            }

            match closest.closest {
                Some((a, b)) => axgeom::CastResult::Hit(CastAnswer { elems: a, mag: b }),
                None => axgeom::CastResult::NoHit,
            }
        }
    }

    impl<'a, T: Aabb + ManySwap> Assert<'a, T> {
        ///Panics if a disconnect is detected between tree and naive queries.
        pub fn assert_raycast(&mut self, ray: axgeom::Ray<T::Num>, mut rtrait: impl RayCast<T>)
        where
            T::Num: core::fmt::Debug,
        {
            let mut res_naive = Vec::new();

            let mut tree = Tree::new(self.inner);
            let mut res_dino = Vec::new();
            match tree.cast_ray(ray, &mut rtrait) {
                axgeom::CastResult::Hit(CastAnswer { elems, mag }) => {
                    for a in elems.into_iter() {
                        let j = crate::assert::into_ptr_usize(a);
                        res_dino.push((j, mag))
                    }
                }
                axgeom::CastResult::NoHit => {
                    //do nothing
                }
            }

            match Naive::new(self.inner).cast_ray(ray, rtrait) {
                axgeom::CastResult::Hit(CastAnswer { elems, mag }) => {
                    for a in elems.into_iter() {
                        let j = crate::assert::into_ptr_usize(a);
                        res_naive.push((j, mag))
                    }
                }
                axgeom::CastResult::NoHit => {
                    //do nothing
                }
            }

            res_naive.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            res_dino.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            assert_eq!(
                res_naive.len(),
                res_dino.len(),
                "len:{:?}",
                (res_naive, res_dino)
            );
            assert!(
                res_naive.iter().eq(res_dino.iter()),
                "nop:\n\n naive:{:?} \n\n broc:{:?}",
                res_naive,
                res_dino
            );
        }
    }
}
///What is returned when the ray hits something.
///It provides the length of the ray,
///as well as all solutions in a unspecified order.
pub struct CastAnswer<'a, T: Aabb> {
    pub elems: Vec<AabbPin<&'a mut T>>,
    pub mag: T::Num,
}
