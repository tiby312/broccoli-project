//!
//! # User guide
//!
//! There are four flavors of the same fundamental raycast api provided in this module.
//! There is a naive version, and there is a version that uses the tree, and there are mutable versions of those
//! that return mutable references.
//!
//!
//! In addition to the tree, the user provides the geometric functions needed by passing an implementation of RayCast.
//! The user must also provide a rectangle within which all objects that the user is interested in possibly
//! being hit by the raycast must include.
//!
//! What is returned is the distance to where the ray cast stopped, plus a list of all bots at that distance.
//! In most cases, only one object is returned, but in the cases where they are ties more can be returned.
//! All possible solutions are returned since it would be hard to define which of the tied objects would be returned.
//! So the Option returns Some() if and only if the list returned has atleast one element in it.
//!
//! # Notes

//! At first the algorithm worked by splitting the ray into two where the ray intersected the divider.
//! So one ray would have the same origin point, and the other would have the point at which the ray
//! intersected the divder as the origin point. The problem with this is that there might not be a clean solution
//! to the new point of the second ray. The point that you compute may not lie exactly on a point along the ray.
//!
//! With real numbers this isnt a problem. There would always be a solution. But real numbers don't exist
//! in the real world. Floating points will be close, but not perfect. If you are using integers, the corner case problems
//! are more apparent.
//!
//! The solution instead was to never subdivide the ray. Its always the same. Instead, keep subdividing the area into rectangles.
//!
//! Why does the user have to provide a finite rectangle up front? The reason is implementation simplicity/performance.
//! By doing this, we don't have to special case the nodes along the outside of the tree.
//! We also don't have have to worry about overflow and underflow problems of providing a rectangle that
//! just barely fits into the number type.
//!

use crate::query::inner_prelude::*;
use axgeom::Ray;

///A Vec<T> is returned since there coule be ties where the ray hits multiple T at a length N away.
//pub type RayCastResult<T, N> = axgeom::CastResult<(Vec<T>, N)>;

///This is the trait that defines raycast specific geometric functions that are needed by this raytracing algorithm.
///By containing all these functions in this trait, we can keep the trait bounds of the underlying Num to a minimum
///of only needing Ord.
pub trait RayCast {
    type T: Aabb<Num = Self::N>;
    type N: Num;

    fn cast_to_aaline<A: Axis>(
        &mut self,
        ray: &Ray<Self::N>,
        line: A,
        val: Self::N,
    ) -> axgeom::CastResult<Self::N>;

    ///Returns true if the ray intersects with this rectangle.
    ///This function allows as to prune which nodes to visit.
    fn cast_broad(
        &mut self,
        ray: &Ray<Self::N>,
        a: PMut<Self::T>,
    ) -> axgeom::CastResult<Self::N>;

    ///The expensive collision detection
    ///This is where the user can do expensive collision detection on the shape
    ///contains within it's bounding box.
    ///Its default implementation just calls cast_broad()
    fn cast_fine(
        &mut self,
        ray: &Ray<Self::N>,
        a: PMut<Self::T>,
    ) -> axgeom::CastResult<Self::N>;
}




pub struct RayCastClosure<T, A, B, C, D, E> {
    pub _p: PhantomData<T>,
    pub acc: A,
    pub broad: B,
    pub fine: C,
    pub xline: D,
    pub yline: E,
}

pub fn from_closure<
    AA:Axis,
    T: Aabb,
    A,
    B,
    C,
    D,
    E,
>( _tree:&Tree<AA,T>,
    acc: A,
    broad: B,
    fine: C,
    xline: D,
    yline: E)->RayCastClosure<T,A,B,C,D,E> 
    where 
        B:FnMut(&mut A, &Ray<T::Num>, PMut<T>) -> CastResult<T::Num>,
        C: FnMut(&mut A, &Ray<T::Num>, PMut<T>) -> CastResult<T::Num>,
        D: FnMut(&mut A, &Ray<T::Num>, T::Num) -> CastResult<T::Num>,
        E: FnMut(&mut A, &Ray<T::Num>, T::Num) -> CastResult<T::Num>{
        RayCastClosure {
            _p: PhantomData,
            acc,
            broad,
            fine,
            xline,
            yline,
        }
}

impl<
        T: Aabb,
        A,
        B,
        C,
        D,
        E,
    > RayCast for RayCastClosure< T, A, B, C, D, E>
where
        B:FnMut(&mut A, &Ray<T::Num>, PMut<T>) -> CastResult<T::Num>,
        C: FnMut(&mut A, &Ray<T::Num>, PMut<T>) -> CastResult<T::Num>,
        D: FnMut(&mut A, &Ray<T::Num>, T::Num) -> CastResult<T::Num>,
        E: FnMut(&mut A, &Ray<T::Num>, T::Num) -> CastResult<T::Num>
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
    ) -> CastResult<Self::N> {
        (self.broad)(&mut self.acc, ray, a)
    }

    fn cast_fine(&mut self, ray: &Ray<Self::N>, a: PMut<Self::T>) -> CastResult<Self::N> {
        (self.fine)(&mut self.acc, ray, a)
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

    ///Returns true if the ray intersects with this rectangle.
    ///This function allows as to prune which nodes to visit.
    fn cast_broad(
        &mut self,
        ray: &Ray<Self::N>,
        a: PMut<Self::T>,
    ) -> axgeom::CastResult<Self::N> {
        self.0.cast_broad(ray, a)
    }

    ///The expensive collision detection
    ///This is where the user can do expensive collision detection on the shape
    ///contains within it's bounding box.
    ///Its default implementation just calls cast_broad()
    fn cast_fine(
        &mut self,
        ray: &Ray<Self::N>,
        a: PMut<Self::T>,
    ) -> axgeom::CastResult<Self::N> {
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
        let y = match raytrait.cast_broad(ray, b.borrow_mut()) {
            axgeom::CastResult::Hit(val) => val,
            axgeom::CastResult::NoHit => {
                return;
            }
        };

        match self.closest.as_mut() {
            Some(dis) => {
                if y > dis.1 {
                    //no way this bot will be a candidate, return.
                    return;
                } else {
                    //this aabb could be a candidate, continue.
                }
            }
            None => {
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
        match self
            .rtrait
            .cast_to_aaline(&self.ray, line.0, line.1)
        {
            axgeom::CastResult::Hit(val) => match self.closest.get_dis() {
                Some(dis) => {
                    if val <= dis {
                        true
                    } else {
                        false
                    }
                }
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

        if let Some(range) = nn.cont {
            //Determine if we should handle this node or not.
            match range.contains_ext(*blap.ray.point.get_axis(axis)) {
                core::cmp::Ordering::Less => blap.should_recurse((axis, range.start)),
                core::cmp::Ordering::Greater => blap.should_recurse((axis, range.end)),
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




use super::NaiveQueries;
impl<K:NaiveQueries> RaycastNaiveQuery for K{}
pub trait RaycastNaiveQuery:NaiveQueries{

    fn raycast_mut<'a>(
        &'a mut self,
        ray: axgeom::Ray<Self::Num>,
        rtrait: &mut impl RayCast<T = Self::T, N = Self::Num>,
    ) -> axgeom::CastResult<(Vec<PMut<'a,Self::T>>, Self::Num)> {
        raycast_naive_mut(self.get_slice_mut(), ray, rtrait)
    }



}

//TODO condense
fn raycast_naive_mut<'a, T: Aabb>(
    bots: PMut<'a, [T]>,
    ray: Ray<T::Num>,
    rtrait: &mut impl RayCast<N = T::Num, T = T>,
) -> axgeom::CastResult<(Vec<PMut<'a, T>>, T::Num)> {
    let mut closest = Closest { closest: None };

    for b in bots.iter_mut() {
        closest.consider(&ray, b, rtrait);
    }

    match closest.closest {
        Some((a, b)) => axgeom::CastResult::Hit((a, b)),
        None => axgeom::CastResult::NoHit,
    }
}





use super::Queries;
impl<'a,K:Queries<'a>> RaycastQuery<'a> for K{}

pub trait RaycastQuery<'a>:Queries<'a>{


/*
/// Find the elements that are hit by a ray.
    ///
    /// The user supplies to functions:
    ///
    /// `fine` is a function that returns the true length of a ray
    /// cast to an object.
    ///
    /// `broad` is a function that returns the length of a ray cast to
    /// a axis aligned rectangle. This function
    /// is used as a conservative estimate to prune out elements which minimizes
    /// how often the `fine` function gets called.
    ///
    /// `border` is the starting axis axis aligned rectangle to use. This
    /// rectangle will be split up and used to prune candidated. All candidate elements
    /// should be within this starting rectangle.
    ///
    /// The result is returned as a `Vec`. In the event of a tie, multiple
    /// elements can be returned.
    ///
    /// `acc` is a user defined object that is passed to every call to either
    /// the `fine` or `broad` functions.
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// use axgeom::{vec2,ray};
    ///
    /// let border = rect(-100,100,-100,100);
    ///
    /// let mut bots = [bbox(rect(0,10,0,10),vec2(5,5)),
    ///                bbox(rect(2,5,2,5),vec2(4,4)),
    ///                bbox(rect(4,10,4,10),vec2(5,5))];
    ///
    /// let mut bots_copy=bots.clone();
    /// let mut tree = broccoli::new(&mut bots);
    /// let ray=ray(vec2(5,-5),vec2(1,2));
    /// let mut counter =0;
    /// let res = tree.raycast_mut(
    ///     ray,&mut counter,
    ///     |c,ray,r|{*c+=1;ray.cast_to_rect(r)},
    ///     |c,ray,t|{*c+=1;ray.cast_to_rect(&t.rect)},   //Do more fine-grained checking here.
    ///     border);
    ///
    /// let (bots,dis)=res.unwrap();
    /// assert_eq!(dis,2);
    /// assert_eq!(bots.len(),1);
    /// assert_eq!(bots[0].inner,vec2(5,5));
    ///```
    */
    /// Companion function to [`Queries::raycast_mut()`] for cases where the use wants to
    /// use the trait instead of closures.
    fn raycast_mut<'b, R: RayCast<T = Self::T, N = Self::Num>>(
        &'b mut self,
        ray: axgeom::Ray<Self::Num>,
        rtrait: &mut R,
    ) -> axgeom::CastResult<(Vec<PMut<'b, Self::T>>, Self::Num)>
    where
        'a: 'b,
    {
        let axis=self.axis();
        let rtrait = RayCastBorrow(rtrait);
        let dt = self.vistr_mut().with_depth(Depth(0));

        let closest = Closest { closest: None };
        let mut blap = Blap {
            rtrait,
            ray,
            closest,
        };
        recc(axis, dt, &mut blap);

        match blap.closest.closest {
            Some((a, b)) => axgeom::CastResult::Hit((a, b)),
            None => axgeom::CastResult::NoHit,
        }
    }
}
