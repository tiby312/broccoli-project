//! Module contains query related structs.

mod inner_prelude {
    pub use crate::inner_prelude::*;
    pub use alloc::vec::Vec;
    pub(crate) use axgeom;
    pub use axgeom::Rect;
    pub use compt::*;
    pub use core::marker::PhantomData;
    pub use itertools::Itertools;
}

pub use graphics::DividerDrawer;
//pub use raycast::RayCastResult;
pub use rect::{MultiRectMut, RectIntersectErr};

pub use crate::query::nbody::NodeMassTrait;

///aabb broadphase collision detection
mod colfind;
pub use colfind::ColMulti;
pub use colfind::NotSortedQueryBuilder;
pub use colfind::QueryBuilder;

///Provides functionality to draw the dividers of [`Tree`].
mod graphics;

///Contains all k_nearest code.
mod k_nearest;
pub use k_nearest::Knearest;
//pub use k_nearest::KnearestClosure;
pub use k_nearest::KResult;
pub use k_nearest::KnearestResult;

///Contains all raycast code.
mod raycast;
//pub use raycast::RayCastClosure;
pub use raycast::RayCast;

///Allows user to intersect the tree with a seperate group of bots.
mod intersect_with;

mod nbody;

///Contains rect code.
mod rect;

///Contains misc tools
mod tools;

use self::inner_prelude::*;

///Queries that can be performed on a tree that is not sorted
///These functions are not documented since they match the same
///behavior as those in the [`Queries`] trait.
pub trait NotSortedQueries<'a> {
    type A: Axis;
    type T: Aabb<Num = Self::Num> + 'a;
    type Num: Num;

    #[must_use]
    fn vistr_mut(&mut self) -> VistrMut<NodeMut<'a, Self::T>>;

    #[must_use]
    fn vistr(&self) -> Vistr<NodeMut<'a, Self::T>>;

    #[must_use]
    fn axis(&self) -> Self::A;

    fn new_colfind_builder<'c>(&'c mut self) -> NotSortedQueryBuilder<'c, 'a, Self::A, Self::T> {
        NotSortedQueryBuilder::new(self.axis(), self.vistr_mut())
    }

    fn find_colliding_pairs_mut(&mut self, mut func: impl FnMut(PMut<Self::T>, PMut<Self::T>)) {
        query::colfind::NotSortedQueryBuilder::new(self.axis(), self.vistr_mut())
            .query_seq(move |a, b| func(a, b));
    }

    fn find_colliding_pairs_mut_par(
        &mut self,
        func: impl Fn(PMut<Self::T>, PMut<Self::T>) + Clone + Send + Sync,
    ) where
        Self::T: Send + Sync,
        Self::Num: Send + Sync,
    {
        query::colfind::NotSortedQueryBuilder::new(self.axis(), self.vistr_mut())
            .query_par(move |a, b| func(a, b));
    }
}

///Query functions. User defines `vistr()` functions, and the query functions
///are automatically provided by this trait.
pub trait Queries<'a> {
    type A: Axis;
    type T: Aabb<Num = Self::Num> + 'a;
    type Num: Num;

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [bbox(rect(0,10,0,10),0)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// use compt::Visitor;
    /// for b in tree.vistr_mut().dfs_preorder_iter().flat_map(|n|n.into_range().iter_mut()){
    ///    *b.unpack_inner()+=1;    
    /// }
    /// assert_eq!(bots[0].inner,1);
    ///```
    #[must_use]
    fn vistr_mut(&mut self) -> VistrMut<NodeMut<'a, Self::T>>;

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// use compt::Visitor;
    /// let mut test = Vec::new();
    /// for b in tree.vistr().dfs_preorder_iter().flat_map(|n|n.range.iter()){
    ///    test.push(b);
    /// }
    /// assert_eq!(test[0],&axgeom::rect(0,10,0,10));
    ///```
    #[must_use]
    fn vistr(&self) -> Vistr<NodeMut<'a, Self::T>>;

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// use axgeom::Axis;
    /// assert!(tree.axis().is_equal_to(broccoli::default_axis()));
    ///```
    #[must_use]
    fn axis(&self) -> Self::A;

    /// Find all aabb intersections and return a PMut<T> of it. Unlike the regular `find_colliding_pairs_mut`, this allows the
    /// user to access a read only reference of the AABB.
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [bbox(rect(0,10,0,10),0u8),bbox(rect(5,15,5,15),0u8)];
    /// let mut tree = broccoli::new(&mut bots);
    /// tree.find_colliding_pairs_mut(|a,b|{
    ///    *a.unpack_inner()+=1;
    ///    *b.unpack_inner()+=1;
    /// });
    ///
    /// assert_eq!(bots[0].inner,1);
    /// assert_eq!(bots[1].inner,1);
    ///```
    fn find_colliding_pairs_mut(&mut self, mut func: impl FnMut(PMut<Self::T>, PMut<Self::T>)) {
        colfind::QueryBuilder::new(self.axis(), self.vistr_mut()).query_seq(move |a, b| func(a, b));
    }

    /// The parallel version of [`Queries::find_colliding_pairs_mut`].
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [bbox(rect(0,10,0,10),0u8),bbox(rect(5,15,5,15),0u8)];
    /// let mut tree = broccoli::new(&mut bots);
    /// tree.find_colliding_pairs_mut_par(|a,b|{
    ///    *a.unpack_inner()+=1;
    ///    *b.unpack_inner()+=1;
    /// });
    ///
    /// assert_eq!(bots[0].inner,1);
    /// assert_eq!(bots[1].inner,1);
    ///```
    fn find_colliding_pairs_mut_par(
        &mut self,
        func: impl Fn(PMut<Self::T>, PMut<Self::T>) + Send + Sync + Clone,
    ) where
        Self::T: Send + Sync,
        Self::Num: Send + Sync,
    {
        colfind::QueryBuilder::new(self.axis(), self.vistr_mut()).query_par(move |a, b| func(a, b));
    }

    /// For analysis, allows the user to query with custom settings
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [bbox(rect(0,10,0,10),0u8),bbox(rect(5,15,5,15),0u8)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// let builder=tree.new_colfind_builder();
    /// let builder=builder.with_switch_height(4);
    /// builder.query_seq(|a,b|{
    ///    *a.unpack_inner()+=1;
    ///    *b.unpack_inner()+=1;
    /// });
    ///
    /// assert_eq!(bots[0].inner,1);
    /// assert_eq!(bots[1].inner,1);
    ///```
    fn new_colfind_builder<'c>(&'c mut self) -> QueryBuilder<'c, 'a, Self::A, Self::T> {
        QueryBuilder::new(self.axis(), self.vistr_mut())
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [rect(0,10,0,10),rect(20,30,20,30)];
    /// let mut tree = broccoli::new(&mut bots);
    /// let mut test = Vec::new();
    /// tree.for_all_intersect_rect(&rect(9,20,9,20),|a|{
    ///    test.push(a);
    /// });
    ///
    /// assert_eq!(test[0],&rect(0,10,0,10));
    ///
    ///```
    fn for_all_intersect_rect<'b>(&'b self, rect: &Rect<Self::Num>, func: impl FnMut(&'b Self::T))
    where
        'a: 'b,
    {
        rect::for_all_intersect_rect(self.axis(), self.vistr(), rect, func);
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [bbox(rect(0,10,0,10),0u8)];
    /// let mut tree = broccoli::new(&mut bots);
    /// tree.for_all_intersect_rect_mut(&rect(9,20,9,20),|a|{
    ///    *a.unpack_inner()+=1;    
    /// });
    ///
    /// assert_eq!(bots[0].inner,1);
    ///
    ///```
    fn for_all_intersect_rect_mut<'b>(
        &'b mut self,
        rect: &Rect<Self::Num>,
        mut func: impl FnMut(PMut<'b, Self::T>),
    ) where
        'a: 'b,
    {
        rect::for_all_intersect_rect_mut(self.axis(), self.vistr_mut(), rect, move |a| (func)(a));
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [rect(0,10,0,10),rect(20,30,20,30)];
    /// let mut tree = broccoli::new(&mut bots);
    /// let mut test = Vec::new();
    /// tree.for_all_in_rect(&rect(0,20,0,20),|a|{
    ///    test.push(a);
    /// });
    ///
    /// assert_eq!(test[0],&rect(0,10,0,10));
    ///
    fn for_all_in_rect<'b>(&'b self, rect: &Rect<Self::Num>, func: impl FnMut(&'b Self::T))
    where
        'a: 'b,
    {
        rect::for_all_in_rect(self.axis(), self.vistr(), rect, func);
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [bbox(rect(0,10,0,10),0u8)];
    /// let mut tree = broccoli::new(&mut bots);
    /// tree.for_all_in_rect_mut(&rect(0,10,0,10),|a|{
    ///    *a.unpack_inner()+=1;    
    /// });
    ///
    /// assert_eq!(bots[0].inner,1);
    ///
    ///```
    fn for_all_in_rect_mut<'b>(
        &'b mut self,
        rect: &Rect<Self::Num>,
        mut func: impl FnMut(PMut<'b, Self::T>),
    ) where
        'a: 'b,
    {
        rect::for_all_in_rect_mut(self.axis(), self.vistr_mut(), rect, move |a| (func)(a));
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots = [bbox(rect(0,10,0,10),0u8)];
    /// let mut tree = broccoli::new(&mut bots);
    /// tree.for_all_not_in_rect_mut(&rect(10,20,10,20),|a|{
    ///    *a.unpack_inner()+=1;    
    /// });
    ///
    /// assert_eq!(bots[0].inner,1);
    ///
    ///```
    fn for_all_not_in_rect_mut<'b>(
        &'b mut self,
        rect: &Rect<Self::Num>,
        mut func: impl FnMut(PMut<'b, Self::T>),
    ) where
        'a: 'b,
    {
        rect::for_all_not_in_rect_mut(self.axis(), self.vistr_mut(), rect, move |a| (func)(a));
    }

    /// If we have two non intersecting rectangles, it is safe to return to the user two sets of mutable references
    /// of the bots strictly inside each rectangle since it is impossible for a bot to belong to both sets.
    ///
    /// # Safety
    ///
    /// Unsafe code is used.  We unsafely convert the references returned by the rect query
    /// closure to have a longer lifetime.
    /// This allows the user to store mutable references of non intersecting rectangles at the same time.
    /// If two requested rectangles intersect, an error is returned.
    ///
    /// Handles a multi rect mut "sessions" within which
    /// the user can query multiple non intersecting rectangles.
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots1 = [bbox(rect(0,10,0,10),0u8)];
    /// let mut tree = broccoli::new(&mut bots1);
    /// let mut multi = tree.multi_rect();
    ///
    /// multi.for_all_in_rect_mut(rect(0,10,0,10),|a|{}).unwrap();
    /// let res = multi.for_all_in_rect_mut(rect(5,15,5,15),|a|{});
    /// assert_eq!(res,Err(broccoli::query::RectIntersectErr));
    ///```
    #[must_use]
    fn multi_rect<'c>(&'c mut self) -> rect::MultiRectMut<'c, 'a, Self::A, Self::T> {
        rect::MultiRectMut::new(self.axis(), self.vistr_mut())
    }

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
    /// let border = rect(0,100,0,100);
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
    #[must_use]
    fn raycast_mut<'b, Acc>(
        &'b mut self,
        ray: axgeom::Ray<Self::Num>,
        acc: &mut Acc,
        broad: impl FnMut(&mut Acc, &Ray<Self::Num>, &Rect<Self::Num>) -> CastResult<Self::Num>,
        fine: impl FnMut(&mut Acc, &Ray<Self::Num>, &Self::T) -> CastResult<Self::Num>,
        border: Rect<Self::Num>,
    ) -> axgeom::CastResult<(Vec<PMut<'b, Self::T>>, Self::Num)>
    where
        'a: 'b,
    {
        let mut rtrait = raycast::RayCastClosure::new(acc, broad, fine);
        raycast::raycast_mut(self.axis(), self.vistr_mut(), border, ray, &mut rtrait)
    }

    /// Companion function to [`Queries::raycast_mut()`] for cases where the use wants to
    /// use the trait instead of closures.
    fn raycast_trait_mut<'b, Acc, R: crate::query::RayCast<T = Self::T, N = Self::Num>>(
        &'b mut self,
        ray: axgeom::Ray<Self::Num>,
        rtrait: R,
        border: Rect<Self::Num>,
    ) -> axgeom::CastResult<(Vec<PMut<'b, Self::T>>, Self::Num)>
    where
        'a: 'b,
    {
        raycast::raycast_mut(self.axis(), self.vistr_mut(), border, ray, rtrait)
    }

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
    #[must_use]
    fn k_nearest_mut<'b, Acc>(
        &'b mut self,
        point: Vec2<Self::Num>,
        num: usize,
        acc: &mut Acc,
        broad: impl FnMut(&mut Acc, Vec2<Self::Num>, &Rect<Self::Num>) -> Self::Num,
        fine: impl FnMut(&mut Acc, Vec2<Self::Num>, &Self::T) -> Self::Num,
        border: Rect<Self::Num>,
    ) -> KResult<Self::T>
    where
        'a: 'b,
    {
        let mut foo = k_nearest::KnearestClosure::new(acc, broad, fine);

        k_nearest::k_nearest_mut(self.axis(), self.vistr_mut(), point, num, &mut foo, border)
    }

    /// Companion function to [`Queries::k_nearest_mut()`] for cases where the use wants to
    /// use the trait instead of closures.
    #[must_use]
    fn k_nearest_trait_mut<'b, Acc, K: query::Knearest<T = Self::T, N = Self::Num>>(
        &'b mut self,
        point: Vec2<Self::Num>,
        num: usize,
        ktrait: K,
        border: Rect<Self::Num>,
    ) -> KResult<Self::T>
    where
        'a: 'b,
    {
        k_nearest::k_nearest_mut(self.axis(), self.vistr_mut(), point, num, ktrait, border)
    }

    /// Find collisions between elements in this tree,
    /// with the specified slice of elements.
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut bots1 = [bbox(rect(0,10,0,10),0u8)];
    /// let mut bots2 = [bbox(rect(5,15,5,15),0u8)];
    /// let mut tree = broccoli::new(&mut bots1);
    ///
    /// tree.intersect_with_mut(&mut bots2,|a,b|{
    ///    *a.unpack_inner()+=1;
    ///    *b.unpack_inner()+=2;    
    /// });
    ///
    /// assert_eq!(bots1[0].inner,1);
    /// assert_eq!(bots2[0].inner,2);
    ///```
    fn intersect_with_mut<X: Aabb<Num = Self::Num>>(
        &mut self,
        other: &mut [X],
        func: impl Fn(PMut<Self::T>, PMut<X>),
    ) {
        intersect_with::intersect_with_mut(self.axis(), self.vistr_mut(), other, move |a, b| {
            (func)(a, b)
        })
    }

    /// # Examples
    ///
    /// ```
    /// use broccoli::{prelude::*,bbox,rect};
    ///
    /// struct Drawer;
    /// impl broccoli::query::DividerDrawer for Drawer{
    ///     type N=i32;
    ///     fn draw_divider<A:axgeom::Axis>(
    ///             &mut self,
    ///             axis:A,
    ///             div:Self::N,
    ///             cont:[Self::N;2],
    ///             length:[Self::N;2],
    ///             depth:usize)
    ///     {
    ///         if axis.is_xaxis(){
    ///             //draw vertical line
    ///         }else{
    ///             //draw horizontal line
    ///         }
    ///     }
    /// }
    ///
    /// let border=rect(0,100,0,100);
    /// let mut bots =[rect(0,10,0,10)];
    /// let tree=broccoli::new(&mut bots);
    /// tree.draw_divider(&mut Drawer,&border);
    /// ```
    ///
    fn draw_divider(
        &self,
        drawer: &mut impl graphics::DividerDrawer<N = Self::Num>,
        rect: &Rect<Self::Num>,
    ) {
        graphics::draw(self.axis(), self.vistr(), drawer, rect)
    }

    ///Experimental. See broccoli demo
    fn nbody_mut<X: query::nbody::NodeMassTrait<Num = Self::Num, Item = Self::T> + Send + Sync>(
        &mut self,
        ncontext: X,
        rect: Rect<Self::Num>,
    ) where
        X::No: Send,
        Self::T: Send + Sync,
        Self::Num: Send + Sync,
    {
        query::nbody::nbody(self.axis(), self.vistr_mut(), ncontext, rect)
    }

    ///Experimental. See broccoli demo
    fn nbody_mut_par<
        X: query::nbody::NodeMassTrait<Num = Self::Num, Item = Self::T> + Sync + Send,
    >(
        &mut self,
        ncontext: X,
        rect: Rect<Self::Num>,
    ) where
        X::No: Send,
        Self::T: Send + Sync,
        Self::Num: Send + Sync,
    {
        query::nbody::nbody_par(self.axis(), self.vistr_mut(), ncontext, rect)
    }
}

///For comparison, the sweep and prune algorithm
pub fn find_collisions_sweep_mut<A: Axis, T: Aabb>(
    bots: &mut [T],
    axis: A,
    mut func: impl FnMut(PMut<T>, PMut<T>),
) {
    colfind::query_sweep_mut(axis, bots, |a, b| func(a, b));
}

///Provides the naive implementation of the [`Tree`] api.
pub struct NaiveAlgs<'a, T> {
    bots: PMut<'a, [T]>,
}

impl<'a, T: Aabb> NaiveAlgs<'a, T> {
    #[must_use]
    pub fn raycast_mut<Acc>(
        &mut self,
        ray: axgeom::Ray<T::Num>,
        start: &mut Acc,
        broad: impl FnMut(&mut Acc, &Ray<T::Num>, &Rect<T::Num>) -> CastResult<T::Num>,
        fine: impl FnMut(&mut Acc, &Ray<T::Num>, &T) -> CastResult<T::Num>,
        border: Rect<T::Num>,
    ) -> axgeom::CastResult<(Vec<PMut<T>>, T::Num)> {
        let mut rtrait = raycast::RayCastClosure {
            a: start,
            broad,
            fine,
            _p: PhantomData,
        };
        raycast::raycast_naive_mut(self.bots.borrow_mut(), ray, &mut rtrait, border)
    }

    #[must_use]
    pub fn k_nearest_mut<Acc>(
        &mut self,
        point: Vec2<T::Num>,
        num: usize,
        start: &mut Acc,
        broad: impl FnMut(&mut Acc, Vec2<T::Num>, &Rect<T::Num>) -> T::Num,
        fine: impl FnMut(&mut Acc, Vec2<T::Num>, &T) -> T::Num,
    ) -> KResult<T> {
        let mut knear = k_nearest::KnearestClosure {
            acc: start,
            broad,
            fine,
            _p: PhantomData,
        };
        k_nearest::k_nearest_naive_mut(self.bots.borrow_mut(), point, num, &mut knear)
    }
}

impl<'a, T: Aabb> NaiveAlgs<'a, T> {
    pub fn for_all_in_rect_mut(&mut self, rect: &Rect<T::Num>, func: impl FnMut(PMut<T>)) {
        rect::naive_for_all_in_rect_mut(self.bots.borrow_mut(), rect, func);
    }
    pub fn for_all_not_in_rect_mut(&mut self, rect: &Rect<T::Num>, func: impl FnMut(PMut<T>)) {
        rect::naive_for_all_not_in_rect_mut(self.bots.borrow_mut(), rect, func);
    }

    pub fn for_all_intersect_rect_mut(&mut self, rect: &Rect<T::Num>, func: impl FnMut(PMut<T>)) {
        rect::naive_for_all_intersect_rect_mut(self.bots.borrow_mut(), rect, func);
    }

    pub fn find_colliding_pairs_mut(&mut self, mut func: impl FnMut(PMut<T>, PMut<T>)) {
        colfind::query_naive_mut(self.bots.borrow_mut(), |a, b| func(a, b));
    }
}

impl<'a, T: Aabb> NaiveAlgs<'a, T> {
    #[must_use]
    pub fn from_slice(a: &'a mut [T]) -> NaiveAlgs<'a, T> {
        NaiveAlgs { bots: PMut::new(a) }
    }
    #[must_use]
    pub fn new(bots: PMut<'a, [T]>) -> NaiveAlgs<'a, T> {
        NaiveAlgs { bots }
    }

    //#[cfg(feature = "nbody")]
    pub fn nbody(&mut self, func: impl FnMut(PMut<T>, PMut<T>)) {
        nbody::naive_mut(self.bots.borrow_mut(), func);
    }
}
