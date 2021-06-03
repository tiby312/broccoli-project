use crate::*;

#[cfg(test)]
mod tests;

pub mod build;



mod query;
pub mod assert{
    use super::query;
    pub use query::raycast::assert_raycast;
    pub use query::knearest::assert_k_nearest_mut;
    pub use query::rect::assert_for_all_in_rect_mut;
    pub use query::rect::assert_for_all_intersect_rect_mut;
    pub use query::rect::assert_for_all_not_in_rect_mut;    
}
pub mod helper{
    use super::query;
    pub use query::raycast::{from_closure as raycast_from_closure,default_rect_raycast};
    pub use query::knearest::{from_closure as knearest_from_closure,default_rect_knearest};
    pub use query::colfind::builder::from_closure as colfind_from_closure;
}
pub use query::draw::DividerDrawer;
pub use query::colfind::builder::{QueryBuilder,NotSortedQueryBuilder,Consumer};
pub use query::raycast::{CastAnswer,RayCast};
pub use query::knearest::{Knearest,KnearestResult};
pub use query::rect::RectIntersectErr;






use build::TreeBuilder;

pub mod container;

type TreeInner<N> = compt::dfs_order::CompleteTreeContainer<N, compt::dfs_order::PreOrder>;

#[repr(transparent)]
struct TreePtr<T: Aabb> {
    _inner: TreeInner<NodePtr<T>>
}

/// A space partitioning tree.
#[repr(transparent)]
pub struct Tree<'a, T: Aabb> {
    inner: TreeInner<Node<'a, T>>
}

///Create a [`Tree`].
///
/// # Examples
///
///```
/// let mut bots = [axgeom::rect(0,10,0,10)];
/// let tree = broccoli::new(&mut bots);
///
///```
pub fn new<T: Aabb>(bots: &mut [T]) -> Tree<T> {
    Tree::new(bots)
}

///Create a [`Tree`] in parallel.
///
/// # Examples
///
///```
/// let mut bots = [axgeom::rect(0,10,0,10)];
/// let tree = broccoli::new_par(broccoli::RayonJoin,&mut bots);
///
///```
pub fn new_par<T: Aabb + Send + Sync>(joiner: impl crate::Joinable, bots: &mut [T]) -> Tree<T>
where
    T::Num: Send + Sync,
{
    Tree::new_par(joiner, bots)
}


impl<'a, T: Aabb> Tree<'a, T> {

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
    #[inline(always)]
    pub fn vistr_mut(&mut self) -> VistrMut<Node<'a, T>> {
        VistrMut::new(self.inner.vistr_mut())
    }


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
    #[inline(always)]
    pub fn vistr(&self) -> Vistr<Node<'a, T>> {
        self.inner.vistr()
    }

    ///Create a [`Tree`].
    ///
    /// # Examples
    ///
    ///```
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let tree = broccoli::Tree::new(&mut bots);
    ///
    ///```
    pub fn new(bots: &'a mut [T]) -> Tree<'a, T> {
        TreeBuilder::new(bots).build_seq()
    }
    ///Create a [`Tree`] in parallel.
    ///
    /// # Examples
    ///
    ///```
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let tree = broccoli::Tree::new_par(broccoli::RayonJoin,&mut bots);
    ///
    ///```
    pub fn new_par(joiner: impl crate::Joinable, bots: &'a mut [T]) -> Tree<'a, T>
    where
        T: Send + Sync,
        T::Num: Send + Sync,
    {
        TreeBuilder::new(bots).build_par(joiner)
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::build;
    /// const NUM_ELEMENT:usize=40;
    /// let mut bots = [axgeom::rect(0,10,0,10);NUM_ELEMENT];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.get_height(),build::TreePreBuilder::new(NUM_ELEMENT).get_height());
    ///```
    ///
    #[must_use]
    #[inline(always)]
    pub fn get_height(&self) -> usize {
        self.inner.get_height()
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::build;
    /// const NUM_ELEMENT:usize=7;
    /// let mut bots = [axgeom::rect(0,10,0,10);NUM_ELEMENT];
    /// let mut tree = broccoli::new(&mut bots);
    /// let inner =tree.into_inner();
    /// assert_eq!(inner.into_nodes().len(),1);
    ///```
    #[must_use]
    pub fn into_inner(
        self,
    ) -> compt::dfs_order::CompleteTreeContainer<Node<'a, T>, compt::dfs_order::PreOrder> {
        self.inner
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::build;
    /// const NUM_ELEMENT:usize=7;
    /// let mut bots = [axgeom::rect(0,10,0,10);NUM_ELEMENT];
    /// let mut tree = broccoli::new(&mut bots);
    /// let inner =tree.into_inner();
    /// let tree=unsafe{broccoli::Tree::from_raw_parts(inner)};
    ///```
    ///
    /// # Safety
    ///
    /// Unsafe, since the user may pass a number of aabbs
    /// that does not reflect the true number of aabbs in
    /// every node.
    ///
    pub unsafe fn from_raw_parts(
        inner: compt::dfs_order::CompleteTreeContainer<Node<'a, T>, compt::dfs_order::PreOrder>,
    ) -> Self {
        Tree { inner}
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::build;
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.num_nodes(),build::TreePreBuilder::new(1).num_nodes());
    ///
    ///```
    #[must_use]
    #[warn(deprecated)]
    #[inline(always)]
    pub fn num_nodes(&self) -> usize {
        self.inner.get_nodes().len()
    }

    /// # Examples
    ///
    ///```
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.get_nodes()[0].range[0], axgeom::rect(0,10,0,10));
    ///
    ///```
    #[must_use]
    pub fn get_nodes(&self) -> &[Node<'a, T>] {
        self.inner.get_nodes()
    }

    /// # Examples
    ///
    ///```
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.get_nodes_mut().get_index_mut(0).range[0], axgeom::rect(0,10,0,10));
    ///
    ///```
    #[must_use]
    pub fn get_nodes_mut(&mut self) -> PMut<[Node<'a, T>]> {
        PMut::new(self.inner.get_nodes_mut())
    }

    /// Return the underlying slice of aabbs in the order sorted during tree construction.
    ///
    /// # Examples
    ///
    ///```
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(*tree.get_elements_mut().get_index_mut(0), axgeom::rect(0,10,0,10));
    ///
    ///```
    #[must_use]
    pub fn get_elements_mut(&mut self) -> PMut<[T]> {
        fn foo<'a, T: Aabb>(mut v: VistrMut<'a, Node<T>>) -> PMut<'a, [T]> {
            let mut new_slice = None;

            let mut siz = 0;
            v.borrow_mut().dfs_preorder(|a| {
                siz += a.range.len();
            });
            v.dfs_preorder(|a| {
                if let Some(s) = new_slice.take() {
                    new_slice = Some(crate::pmut::combine_slice(s, a.into_range()));
                } else {
                    new_slice = Some(a.into_range());
                }
            });
            new_slice.unwrap()
        }

        foo(self.vistr_mut())
    }

    /// Return the underlying slice of aabbs in the order sorted during tree construction.
    ///
    /// # Examples
    ///
    ///```
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.get_elements()[0], axgeom::rect(0,10,0,10));
    ///
    ///```
    #[must_use]
    pub fn get_elements(&self) -> &[T] {
        fn foo<'a, T: Aabb>(v: Vistr<'a, Node<T>>) -> &'a [T] {
            let mut new_slice = None;

            v.dfs_preorder(|a| {
                if let Some(s) = new_slice.take() {
                    new_slice = Some(crate::util::combine_slice(s, &a.range));
                } else {
                    new_slice = Some(&a.range);
                }
            });
            new_slice.unwrap()
        }

        foo(self.vistr())
    }

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
    pub fn find_colliding_pairs_mut(&mut self, mut func: impl FnMut(PMut<T>, PMut<T>)) {
        query::colfind::builder::QueryBuilder::new(self.vistr_mut()).query_seq(move |a, b| func(a, b));
    }

    /// The parallel version of [`ColfindQuery::find_colliding_pairs_mut`].
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::{prelude::*,bbox,rect,RayonJoin};
    /// let mut bots = [bbox(rect(0,10,0,10),0u8),bbox(rect(5,15,5,15),0u8)];
    /// let mut tree = broccoli::new(&mut bots);
    /// tree.find_colliding_pairs_mut_par(RayonJoin,|a,b|{
    ///    *a.unpack_inner()+=1;
    ///    *b.unpack_inner()+=1;
    /// });
    ///
    /// assert_eq!(bots[0].inner,1);
    /// assert_eq!(bots[1].inner,1);
    ///```
    pub fn find_colliding_pairs_mut_par(
        &mut self,
        joiner: impl crate::Joinable,
        func: impl Fn(PMut<T>, PMut<T>) + Send + Sync + Clone,
    ) where
        T: Send + Sync,
        T::Num: Send + Sync,
    {
        query::colfind::builder::QueryBuilder::new(self.vistr_mut()).query_par(joiner, move |a, b| func(a, b));
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
    pub fn new_colfind_builder<'c>(&'c mut self) -> query::colfind::builder::QueryBuilder<'c, 'a, T> {
        query::colfind::builder::QueryBuilder::new(self.vistr_mut())
    }

    /// # Examples
    ///
    /// ```
    /// use broccoli::{prelude::*,bbox,rect};
    /// use axgeom::Rect;
    ///
    /// let dim=rect(0,100,0,100);
    /// let mut bots =[rect(0,10,0,10)];
    /// let tree=broccoli::new(&mut bots);
    ///
    /// let mut rects=Vec::new();
    /// tree.draw_divider(
    ///     |axis,node,rect,_|
    ///     {
    ///         if !node.range.is_empty(){    
    ///             rects.push(
    ///                 axis.map_val(
    ///                     Rect {x: node.cont.into(),y: rect.y.into()},
    ///                     Rect {x: rect.x.into(),y: node.cont.into()}
    ///                 )   
    ///             );
    ///         }
    ///     },
    ///     dim
    /// );
    ///
    /// //rects now contains a bunch of rectangles that can be drawn to visualize
    /// //where all the dividers are and how thick they each are.
    ///
    /// ```
    ///
    pub fn draw_divider(
        &self,
        line: impl FnMut(AxisDyn, &Node<T>, &Rect<T::Num>, usize),
        rect: Rect<T::Num>,
    ) {
        use core::marker::PhantomData;
        let mut d = query::draw::DrawClosure {
            _p: PhantomData,
            line,
        };

        query::draw::draw(default_axis(), self.vistr(), &mut d, rect)
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
    pub fn intersect_with_mut<X: Aabb<Num = T::Num>>(
        &mut self,
        other: &mut [X],
        func: impl Fn(PMut<T>, PMut<X>),
    ) {
        //TODO instead of create just a list of BBox, construct a tree using the dividors of the current tree.
        //This way we can parallelize this function.
        //Find all intersecting pairs between the elements in this tree, and the specified elements.
        //No intersecting pairs within each group are looked for, only those between the two groups.
        //For best performance the group that this tree is built around should be the bigger of the two groups.
        //Since the dividers of the tree are used to divide and conquer the problem.
        //If the other group is bigger, consider building the DinoTree around that group instead, and
        //leave this group has a list of bots.
        //
        //Currently this is implemented naively using for_all_intersect_rect_mut().
        //But using the api, it is possible to build up a tree using the current trees dividers
        //to exploit the divide and conquer properties of this problem.
        //The two trees could be recursed at the same time to break up the problem.

        for mut i in PMut::new(other).iter_mut() {
            let rect = *i.get();
            self.for_all_intersect_rect_mut(&rect, |a| {
                func(a, i.borrow_mut());
            });
        }
    }

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
    /// let mut handler = broccoli::helper::knearest_from_closure(
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
    pub fn k_nearest_mut<'b, K: query::knearest::Knearest<T = T, N = T::Num>>(
        &'b mut self,
        point: Vec2<K::N>,
        num: usize,
        ktrait: &mut K,
    ) -> query::knearest::KResult<'b,K::T>
    {
        query::knearest::knearest_mut(self,point,num,ktrait)
    }


    ///Perform nbody
    ///The tree is taken by value so that its nodes can be expended to include more data.
    pub fn nbody_mut_par<N: query::nbody::Nbody<T=T,N=T::Num>>(
        self,
        joiner: impl crate::Joinable,
        no: &mut N,
    ) -> Self
    where
        N: Send + Sync + Splitter,
        N::T: Send + Sync,
        <N::T as Aabb>::Num: Send + Sync,
        N::Mass: Send + Sync,
    {
        query::nbody::nbody_mut_par(self,joiner,no)
    }

    ///Perform nbody
    ///The tree is taken by value so that its nodes can be expended to include more data.
    pub fn nbody_mut<N: query::nbody::Nbody<T=T,N=T::Num>>(self, no: &mut N) -> Self {
        query::nbody::nbody_mut(self,no)
    }

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
    /// let mut handler = broccoli::helper::raycast_from_closure(
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
    pub fn raycast_mut<'b, R: query::raycast::RayCast<T = T, N = T::Num>>(
        &'b mut self,
        ray: axgeom::Ray<T::Num>,
        rtrait: &mut R,
    ) -> axgeom::CastResult<query::raycast::CastAnswer<'b, T>>
    {
        query::raycast::raycast_mut(self,ray,rtrait)
        
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
    pub fn for_all_intersect_rect<'b>(&'b self, rect: &Rect<T::Num>, func: impl FnMut(&'b T))
    {
        query::rect::for_all_intersect_rect(default_axis(), self.vistr(), rect, func);
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
    pub fn for_all_intersect_rect_mut<'b>(
        &'b mut self,
        rect: &Rect<T::Num>,
        mut func: impl FnMut(PMut<'b, T>),
    ){
        query::rect::for_all_intersect_rect_mut(default_axis(), self.vistr_mut(), rect, move |a| {
            (func)(a)
        });
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
    pub fn for_all_in_rect<'b>(&'b self, rect: &Rect<T::Num>, func: impl FnMut(&'b T))
    {
        query::rect::for_all_in_rect(default_axis(), self.vistr(), rect, func);
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
    pub fn for_all_in_rect_mut<'b>(
        &'b mut self,
        rect: &Rect<T::Num>,
        mut func: impl FnMut(PMut<'b, T>),
    ){
        query::rect::for_all_in_rect_mut(default_axis(), self.vistr_mut(), rect, move |a| (func)(a));
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
    pub fn for_all_not_in_rect_mut<'b>(
        &'b mut self,
        rect: &Rect<T::Num>,
        mut func: impl FnMut(PMut<'b, T>),
    )
    {
        query::rect::for_all_not_in_rect_mut(default_axis(), self.vistr_mut(), rect, move |a| (func)(a));
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
    /// assert_eq!(res,Err(broccoli::RectIntersectErr));
    ///```
    #[must_use]
    pub fn multi_rect<'c>(&'c mut self) -> query::rect::MultiRect<'c, 'a, T> {
        query::rect::MultiRect::new(self.vistr_mut())
    }
}








///A version of Tree where the elements are not sorted along each axis, like a KD Tree.
/// For comparison, a normal kd-tree is provided by [`NotSorted`]. In this tree, the elements are not sorted
/// along an axis at each level. Construction of [`NotSorted`] is faster than [`Tree`] since it does not have to
/// sort bots that belong to each node along an axis. But most query algorithms can usually take advantage of this
/// extra property to be faster.
pub struct NotSorted<'a, T: Aabb>(pub Tree<'a, T>);

impl<'a, T: Aabb> NotSorted<'a, T> {
    pub fn new(bots: &'a mut [T]) -> NotSorted<'a, T> {
        TreeBuilder::new(bots).build_not_sorted_seq()
    }

    pub fn new_par(joiner: impl crate::Joinable, bots: &'a mut [T]) -> NotSorted<'a, T>
    where
        T: Send + Sync,
        T::Num: Send + Sync,
    {
        TreeBuilder::new(bots).build_not_sorted_par(joiner)
    }

    #[inline(always)]
    pub fn vistr_mut(&mut self) -> VistrMut<Node<'a, T>> {
        self.0.vistr_mut()
    }

    #[inline(always)]
    pub fn vistr(&self) -> Vistr<Node<'a, T>> {
        self.0.vistr()
    }
    #[inline(always)]
    pub fn get_height(&self) -> usize {
        self.0.get_height()
    }


    pub fn new_colfind_builder<'c>(&'c mut self) -> NotSortedQueryBuilder<'c, 'a, T> {
        NotSortedQueryBuilder::new(self.vistr_mut())
    }

    pub fn find_colliding_pairs_mut(&mut self, mut func: impl FnMut(PMut<T>, PMut<T>)) {
        NotSortedQueryBuilder::new(self.vistr_mut()).query_seq(move |a, b| func(a, b));
    }

    pub fn find_colliding_pairs_mut_par(
        &mut self,
        joiner: impl crate::Joinable,
        func: impl Fn(PMut<T>, PMut<T>) + Clone + Send + Sync,
    ) where
        T: Send + Sync,
        T::Num: Send + Sync,
    {
        NotSortedQueryBuilder::new(self.vistr_mut()).query_par(joiner, move |a, b| func(a, b));
    }




}

