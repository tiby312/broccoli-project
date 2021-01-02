//! Raycast query module

use crate::query::inner_prelude::*;
use axgeom::Ray;

///Create a handler that just casts directly to the axis aligned rectangle
pub fn default_rect_raycast<T: Aabb>(tree: &Tree<T>) -> impl RayCast<T = T, N = T::Num>
where
    T::Num: core::fmt::Debug + num_traits::Signed,
{
    from_closure(
        tree,
        (),
        |_, _, _| None,
        |_, ray, a| ray.cast_to_rect(a.get()),
        |_, ray, val| ray.cast_to_aaline(axgeom::XAXIS, val),
        |_, ray, val| ray.cast_to_aaline(axgeom::YAXIS, val),
    )
}

///A Vec<T> is returned since there coule be ties where the ray hits multiple T at a length N away.
//pub type RayCastResult<T, N> = axgeom::CastResult<(Vec<T>, N)>;

///This is the trait that defines raycast specific geometric functions that are needed by this raytracing algorithm.
///By containing all these functions in this trait, we can keep the trait bounds of the underlying Num to a minimum
///of only needing Ord.
pub trait RayCast {
    type T: Aabb<Num = Self::N>;
    type N: Num;

    ///Return the cast result to a axis aligned line of infinite length.
    fn cast_to_aaline<A: Axis>(
        &mut self,
        ray: &Ray<Self::N>,
        line: A,
        val: Self::N,
    ) -> axgeom::CastResult<Self::N>;

    ///Return the cast result that is cheap and overly conservative.
    ///It may be that the precise cast is fast enough, in which case you can simply
    ///return None. If None is desired, every call to this function for a particular element must
    ///always return None.
    fn cast_broad(
        &mut self,
        ray: &Ray<Self::N>,
        a: PMut<Self::T>,
    ) -> Option<axgeom::CastResult<Self::N>>;

    ///Return the exact cast result.
    fn cast_fine(&mut self, ray: &Ray<Self::N>, a: PMut<Self::T>) -> axgeom::CastResult<Self::N>;
}

use crate::Tree;

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
pub fn from_closure<A, T: Aabb>(
    _tree: &Tree<T>,
    acc: A,
    broad: impl FnMut(&mut A, &Ray<T::Num>, PMut<T>) -> Option<CastResult<T::Num>>,
    fine: impl FnMut(&mut A, &Ray<T::Num>, PMut<T>) -> CastResult<T::Num>,
    xline: impl FnMut(&mut A, &Ray<T::Num>, T::Num) -> CastResult<T::Num>,
    yline: impl FnMut(&mut A, &Ray<T::Num>, T::Num) -> CastResult<T::Num>,
) -> impl RayCast<T = T, N = T::Num> {
    struct RayCastClosure<T, A, B, C, D, E> {
        _p: PhantomData<T>,
        acc: A,
        broad: B,
        fine: C,
        xline: D,
        yline: E,
    }

    impl<T: Aabb, A, B, C, D, E> RayCast for RayCastClosure<T, A, B, C, D, E>
    where
        B: FnMut(&mut A, &Ray<T::Num>, PMut<T>) -> Option<CastResult<T::Num>>,
        C: FnMut(&mut A, &Ray<T::Num>, PMut<T>) -> CastResult<T::Num>,
        D: FnMut(&mut A, &Ray<T::Num>, T::Num) -> CastResult<T::Num>,
        E: FnMut(&mut A, &Ray<T::Num>, T::Num) -> CastResult<T::Num>,
    {
        type T = T;
        type N = T::Num;

        fn cast_to_aaline<X: Axis>(
            &mut self,
            ray: &Ray<Self::N>,
            line: X,
            val: Self::N,
        ) -> axgeom::CastResult<Self::N> {
            if line.is_xaxis() {
                (self.xline)(&mut self.acc, ray, val)
            } else {
                (self.yline)(&mut self.acc, ray, val)
            }
        }
        fn cast_broad(
            &mut self,
            ray: &Ray<Self::N>,
            a: PMut<Self::T>,
        ) -> Option<CastResult<Self::N>> {
            (self.broad)(&mut self.acc, ray, a)
        }

        fn cast_fine(&mut self, ray: &Ray<Self::N>, a: PMut<Self::T>) -> CastResult<Self::N> {
            (self.fine)(&mut self.acc, ray, a)
        }
    }

    RayCastClosure {
        _p: PhantomData,
        acc,
        broad,
        fine,
        xline,
        yline,
    }
}

struct RayCastBorrow<'a, R>(&'a mut R);

impl<'a, R: RayCast> RayCast for RayCastBorrow<'a, R> {
    type T = R::T;
    type N = R::N;
    fn cast_to_aaline<A: Axis>(
        &mut self,
        ray: &Ray<Self::N>,
        line: A,
        val: Self::N,
    ) -> axgeom::CastResult<Self::N> {
        self.0.cast_to_aaline(ray, line, val)
    }

    fn cast_broad(
        &mut self,
        ray: &Ray<Self::N>,
        a: PMut<Self::T>,
    ) -> Option<axgeom::CastResult<Self::N>> {
        self.0.cast_broad(ray, a)
    }

    fn cast_fine(&mut self, ray: &Ray<Self::N>, a: PMut<Self::T>) -> axgeom::CastResult<Self::N> {
        self.0.cast_fine(ray, a)
    }
}

struct Closest<'a, T: Aabb> {
    closest: Option<(Vec<PMut<'a, T>>, T::Num)>,
}
impl<'a, T: Aabb> Closest<'a, T> {
    fn consider<R: RayCast<N = T::Num, T = T>>(
        &mut self,
        ray: &Ray<T::Num>,
        mut b: PMut<'a, T>,
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
        match &self.closest {
            Some(x) => Some(x.1),
            None => None,
        }
    }
}

struct Blap<'a, R: RayCast> {
    rtrait: R,
    ray: Ray<R::N>,
    closest: Closest<'a, R::T>,
}
impl<'a, R: RayCast> Blap<'a, R> {
    fn should_recurse<A: Axis>(&mut self, line: (A, R::N)) -> bool {
        match self.rtrait.cast_to_aaline(&self.ray, line.0, line.1) {
            axgeom::CastResult::Hit(val) => match self.closest.get_dis() {
                Some(dis) => val <= dis,
                None => true,
            },
            axgeom::CastResult::NoHit => false,
        }
    }
}

//Returns the first object that touches the ray.
fn recc<'a, 'b: 'a, A: Axis, T: Aabb, R: RayCast<N = T::Num, T = T>>(
    axis: A,
    stuff: LevelIter<VistrMut<'a, Node<'b, T>>>,
    blap: &mut Blap<'a, R>,
) {
    let ((_depth, nn), rest) = stuff.next();
    let handle_curr = if let Some([left, right]) = rest {
        let axis_next = axis.next();

        let div = match nn.div {
            Some(b) => b,
            None => return,
        };

        let line = (axis, div);

        //more likely to find closest in child than curent node.
        //so recurse first before handling this node.
        if *blap.ray.point.get_axis(axis) < div {
            recc(axis_next, left, blap);

            if blap.should_recurse(line) {
                recc(axis_next, right, blap);
            }
        } else {
            recc(axis_next, right, blap);

            if blap.should_recurse(line) {
                recc(axis_next, left, blap);
            }
        }

        if !nn.range.is_empty() {
            //Determine if we should handle this node or not.
            match nn.cont.contains_ext(*blap.ray.point.get_axis(axis)) {
                core::cmp::Ordering::Less => blap.should_recurse((axis, nn.cont.start)),
                core::cmp::Ordering::Greater => blap.should_recurse((axis, nn.cont.end)),
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
            blap.closest.consider(&blap.ray, b, &mut blap.rtrait);
        }
    }
}

use crate::container::TreeRef;
///Panics if a disconnect is detected between tree and naive queries.
pub fn assert_raycast<T: Aabb>(
    tree: &mut TreeRef<T>,
    ray: axgeom::Ray<T::Num>,
    rtrait: &mut impl RayCast<T = T, N = T::Num>,
) where
    T::Num: core::fmt::Debug,
{
    let bots = tree.get_bbox_elements_mut();
    fn into_ptr_usize<T>(a: &T) -> usize {
        a as *const T as usize
    }
    let mut res_naive = Vec::new();
    match raycast_naive_mut(bots, ray, rtrait) {
        axgeom::CastResult::Hit(CastAnswer { elems, mag }) => {
            for a in elems.into_iter() {
                let r = *a.get();
                let j = into_ptr_usize(a.into_ref());
                res_naive.push((j, r, mag))
            }
        }
        axgeom::CastResult::NoHit => {
            //do nothing
        }
    }

    let mut res_dino = Vec::new();
    match tree.raycast_mut(ray, rtrait) {
        axgeom::CastResult::Hit(CastAnswer { elems, mag }) => {
            for a in elems.into_iter() {
                let r = *a.get();
                let j = into_ptr_usize(a.into_ref());
                res_dino.push((j, r, mag))
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

///Naive implementation
pub fn raycast_naive_mut<'a, T: Aabb>(
    bots: PMut<'a, [T]>,
    ray: Ray<T::Num>,
    rtrait: &mut impl RayCast<N = T::Num, T = T>,
) -> axgeom::CastResult<CastAnswer<'a, T>> {
    let mut closest = Closest { closest: None };

    for b in bots.iter_mut() {
        closest.consider(&ray, b, rtrait);
    }

    match closest.closest {
        Some((a, b)) => axgeom::CastResult::Hit(CastAnswer { elems: a, mag: b }),
        None => axgeom::CastResult::NoHit,
    }
}

use super::Queries;

///What is returned when the ray hits something.
///It provides the length of the ray,
///as well as all solutions in a unspecified order.
pub struct CastAnswer<'a, T: Aabb> {
    pub elems: Vec<PMut<'a, T>>,
    pub mag: T::Num,
}
///Raycast functions that can be called on a tree.
pub trait RaycastQuery<'a>: Queries<'a> {
    /// Find the elements that are hit by a ray.
    ///
    /// The result is returned as a `Vec`. In the event of a tie, multiple
    /// elements can be returned.
    ///
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// use axgeom::{vec2,ray};
    ///
    ///
    /// let mut bots = [bbox(rect(0,10,0,10),vec2(5,5)),
    ///                bbox(rect(2,5,2,5),vec2(4,4)),
    ///                bbox(rect(4,10,4,10),vec2(5,5))];
    ///
    /// let mut bots_copy=bots.clone();
    /// let mut tree = broccoli::new(&mut bots);
    /// let ray=ray(vec2(5,-5),vec2(1,2));
    ///
    /// let mut handler = broccoli::query::raycast::from_closure(
    ///    &tree,
    ///    (),
    ///    |_, _, _| None,
    ///    |_, ray, a| ray.cast_to_rect(&a.rect),
    ///    |_, ray, val| ray.cast_to_aaline(axgeom::XAXIS, val),
    ///    |_, ray, val| ray.cast_to_aaline(axgeom::YAXIS, val),
    /// );
    /// let res = tree.raycast_mut(
    ///     ray,
    ///     &mut handler);
    ///
    /// let res=res.unwrap();
    /// assert_eq!(res.mag,2);
    /// assert_eq!(res.elems.len(),1);
    /// assert_eq!(res.elems[0].inner,vec2(5,5));
    ///```
    fn raycast_mut<'b, R: RayCast<T = Self::T, N = Self::Num>>(
        &'b mut self,
        ray: axgeom::Ray<Self::Num>,
        rtrait: &mut R,
    ) -> axgeom::CastResult<CastAnswer<'b, Self::T>>
    where
        'a: 'b,
    {
        let rtrait = RayCastBorrow(rtrait);
        let dt = self.vistr_mut().with_depth(Depth(0));

        let closest = Closest { closest: None };
        let mut blap = Blap {
            rtrait,
            ray,
            closest,
        };
        recc(default_axis(), dt, &mut blap);

        match blap.closest.closest {
            Some((a, b)) => axgeom::CastResult::Hit(CastAnswer { elems: a, mag: b }),
            None => axgeom::CastResult::NoHit,
        }
    }
}
