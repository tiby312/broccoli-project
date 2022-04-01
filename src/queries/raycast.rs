//! Raycast query module

use super::*;
use axgeom::Ray;



pub struct RayCastBuilder<'a,N,F>{
    ray:Ray<N>,
    floop:&'a mut F
}
impl<'a,T:Aabb,F:Floop<'a,T=T>> RayCastBuilder<'a,T::Num,F>{
    pub fn new(a:&'a mut F,ray:Ray<T::Num>)->Self{
        unimplemented!()
    }

    pub fn default(self)->axgeom::CastResult<CastAnswer<'a, T>>{
        unimplemented!()
    }

    pub fn with_closure(self)->axgeom::CastResult<CastAnswer<'a,T>>{
        unimplemented!();
    }

    pub fn from_trait(self)->axgeom::CastResult<CastAnswer<'a,T>>{
        unimplemented!()
    }
}





///Create a handler that just casts directly to the axis aligned rectangle
pub fn default_rect_raycast<'a,F:Floop<'a,T=T>, T: Aabb>(
    ray:Ray<T::Num>,
    tree: &'a mut F,
) -> axgeom::CastResult<CastAnswer<'a, T>>
where
    T::Num: core::fmt::Debug + num_traits::Signed,
{
    from_closure(
        tree,
        ray,
        (),
        |_, _, _| None,
        |_, ray, a: HalfPin<&mut T>| ray.cast_to_rect(a.get()),
        |_, ray, val| ray.cast_to_aaline(axgeom::XAXIS, val),
        |_, ray, val| ray.cast_to_aaline(axgeom::YAXIS, val),
    )
}

///A Vec<T> is returned since there coule be ties where the ray hits multiple T at a length N away.
//pub type RayCastResult<T, N> = axgeom::CastResult<(Vec<T>, N)>;

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
        a: HalfPin<&mut T>,
    ) -> Option<axgeom::CastResult<T::Num>>;

    ///Return the exact cast result.
    fn cast_fine(&mut self, ray: &Ray<T::Num>, a: HalfPin<&mut T>) -> axgeom::CastResult<T::Num>;
}

use crate::Tree;



pub trait Floop<'a>{
    type T:Aabb;
    fn build<R:RayCast<Self::T>>(&mut self,ray:Ray<<Self::T as Aabb>::Num>,a:R)->axgeom::CastResult<CastAnswer<'a, Self::T>>;
}


impl<'a,T:Aabb> Floop<'a> for HalfPin<&'a mut [T]>{
    type T=T;
    fn build<R:RayCast<Self::T>>(&mut self,ray:Ray<T::Num>,a:R)->axgeom::CastResult<CastAnswer<'a, T>>{
        unimplemented!();
    }
}
impl<'a,T:Aabb> Floop<'a> for Tree<'a,T>{
    type T=T;
    fn build<R:RayCast<Self::T>>(&mut self,ray:Ray<T::Num>,a:R)->axgeom::CastResult<CastAnswer<'a, T>>{
        unimplemented!();
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
pub fn from_closure<'a,F:Floop<'a,T=T>, T: Aabb, A>(
    tree: &mut F,
    ray:Ray<T::Num>,
    acc: A,
    broad: impl FnMut(&mut A, &Ray<T::Num>, HalfPin<&mut T>) -> Option<CastResult<T::Num>>,
    fine: impl FnMut(&mut A, &Ray<T::Num>, HalfPin<&mut T>) -> CastResult<T::Num>,
    xline: impl FnMut(&mut A, &Ray<T::Num>, T::Num) -> CastResult<T::Num>,
    yline: impl FnMut(&mut A, &Ray<T::Num>, T::Num) -> CastResult<T::Num>,
) -> axgeom::CastResult<CastAnswer<'a, T>>{
    struct RayCastClosure<A, B, C, D, E> {
        acc: A,
        broad: B,
        fine: C,
        xline: D,
        yline: E,
    }

    impl<T: Aabb, A, B, C, D, E> RayCast<T> for RayCastClosure<A, B, C, D, E>
    where
        B: FnMut(&mut A, &Ray<T::Num>, HalfPin<&mut T>) -> Option<CastResult<T::Num>>,
        C: FnMut(&mut A, &Ray<T::Num>, HalfPin<&mut T>) -> CastResult<T::Num>,
        D: FnMut(&mut A, &Ray<T::Num>, T::Num) -> CastResult<T::Num>,
        E: FnMut(&mut A, &Ray<T::Num>, T::Num) -> CastResult<T::Num>,
    {
        fn cast_to_aaline<X: Axis>(
            &mut self,
            ray: &Ray<T::Num>,
            line: X,
            val: T::Num,
        ) -> axgeom::CastResult<T::Num> {
            if line.is_xaxis() {
                (self.xline)(&mut self.acc, ray, val)
            } else {
                (self.yline)(&mut self.acc, ray, val)
            }
        }
        fn cast_broad(
            &mut self,
            ray: &Ray<T::Num>,
            a: HalfPin<&mut T>,
        ) -> Option<CastResult<T::Num>> {
            (self.broad)(&mut self.acc, ray, a)
        }

        fn cast_fine(&mut self, ray: &Ray<T::Num>, a: HalfPin<&mut T>) -> CastResult<T::Num> {
            (self.fine)(&mut self.acc, ray, a)
        }
    }

    let r=RayCastClosure {
            acc,
            broad,
            fine,
            xline,
            yline,
        };

    tree.build(ray,r)
}

///Hide the lifetime behind the RayCast trait
///to make things simpler
struct RayCastBorrow<'a, R>(&'a mut R);

impl<'a, T: Aabb, R: RayCast<T>> RayCast<T> for RayCastBorrow<'a, R> {
    fn cast_to_aaline<A: Axis>(
        &mut self,
        ray: &Ray<T::Num>,
        line: A,
        val: T::Num,
    ) -> axgeom::CastResult<T::Num> {
        self.0.cast_to_aaline(ray, line, val)
    }

    fn cast_broad(
        &mut self,
        ray: &Ray<T::Num>,
        a: HalfPin<&mut T>,
    ) -> Option<axgeom::CastResult<T::Num>> {
        self.0.cast_broad(ray, a)
    }

    fn cast_fine(&mut self, ray: &Ray<T::Num>, a: HalfPin<&mut T>) -> axgeom::CastResult<T::Num> {
        self.0.cast_fine(ray, a)
    }
}

struct Closest<'a, T: Aabb> {
    closest: Option<(Vec<HalfPin<&'a mut T>>, T::Num)>,
}
impl<'a, T: Aabb> Closest<'a, T> {
    fn consider<R: RayCast<T>>(
        &mut self,
        ray: &Ray<T::Num>,
        mut b: HalfPin<&'a mut T>,
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
    fn recc<'b: 'a, A: Axis>(&mut self, axis: A, stuff: LevelIter<VistrMut<'a, Node<'b, T>>>) {
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
                    core::cmp::Ordering::Greater => self.should_recurse((axis, nn.cont.end)),
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

///Panics if a disconnect is detected between tree and naive queries.
pub fn assert_raycast<T: Aabb>(
    bots: &mut [T],
    ray: axgeom::Ray<T::Num>,
    rtrait: &mut impl RayCast<T>,
) where
    T::Num: core::fmt::Debug,
{
    fn into_ptr_usize<T>(a: &T) -> usize {
        a as *const T as usize
    }
    let mut res_naive = Vec::new();

    let mut tree = crate::new(bots);
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


    match raycast_naive_mut(HalfPin::new(bots), ray, rtrait) {
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
    bots: HalfPin<&'a mut [T]>,
    ray: Ray<T::Num>,
    rtrait: &mut impl RayCast<T>,
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

pub struct Raycaster<'a, 'b, T: Aabb, R: RayCast<T>> {
    tree: &'b mut Tree<'a, T>,
    rtrait: R,
}
impl<'a, 'b, T: Aabb, R: RayCast<T>> Raycaster<'a, 'b, T, R> {
    pub fn new(tree: &'b mut Tree<'a, T>, rtrait: R) -> Self {
        Raycaster { tree, rtrait }
    }
    pub fn build(mut self, ray: axgeom::Ray<T::Num>) -> axgeom::CastResult<CastAnswer<'b, T>> {
        let rtrait = RayCastBorrow(&mut self.rtrait);
        let dt = self.tree.vistr_mut().with_depth(Depth(0));

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

///What is returned when the ray hits something.
///It provides the length of the ray,
///as well as all solutions in a unspecified order.
pub struct CastAnswer<'a, T: Aabb> {
    pub elems: Vec<HalfPin<&'a mut T>>,
    pub mag: T::Num,
}

pub fn raycast_mut<'b, T: Aabb, R: RayCast<T>>(
    tree: &'b mut Tree<T>,
    ray: axgeom::Ray<T::Num>,
    rtrait: &mut R,
) -> axgeom::CastResult<CastAnswer<'b, T>> {
    let rtrait = RayCastBorrow(rtrait);
    let dt = tree.vistr_mut().with_depth(Depth(0));

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
