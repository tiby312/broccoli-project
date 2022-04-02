//! Broccoli is a broad-phase collision detection library.
//! The base data structure is a hybrid between a [KD Tree](https://en.wikipedia.org/wiki/K-d_tree) and [Sweep and Prune](https://en.wikipedia.org/wiki/Sweep_and_prune).
//!
//! ### Data Structure
//!
//! Using this crate, the user can create three flavors of the same fundamental data structure.
//! The different characteristics are explored more in depth in the [broccoli book](https://tiby312.github.io/broccoli_report)
//!
//! - `(Rect<N>,&mut T)` Semi-direct
//! - `(Rect<N>,T)` Direct
//! - `&mut (Rect<N>,T)` Indirect
//!
//! ### There are so many Tree types which one do I use?
//!
//! The [`container`] module lists the tree types and they are all described there, but in general
//! use [`Tree`] unless you want
//! to use functions like [`collect_colliding_pairs`](crate::container::TreeInd::collect_colliding_pairs).
//! In which case use [`TreeInd`](crate::container::TreeInd).
//!
//! Checkout the github [examples](https://github.com/tiby312/broccoli/tree/master/examples).
//!
//! ### Parallelism
//!
//! Parallel versions of construction and colliding pair finding functions
//! are provided. They use [rayon](https://crates.io/crates/rayon) under the hood which uses work stealing to
//! parallelize divide and conquer style recursive functions.
//!
//! ### Floating Point
//!
//! Broccoli only requires `PartialOrd` for its number type. Instead of panicking on comparisons
//! it doesn't understand, it will just arbitrary pick a result. So if you use regular float primitive types
//! and there is even just one `NaN`, tree construction and querying will not panic,
//! but would have unspecified results.
//! If using floats, it's the users responsibility to not pass `NaN` values into the tree.
//! There is no static protection against this, though if this is desired you can use
//! the [ordered-float](https://crates.io/crates/ordered-float) crate. The Ord trait was not
//! enforced to give users the option to use primitive floats directly which can be easier to
//! work with.
//!
//! ### Protecting Invariants Statically
//!
//! A lot is done to forbid the user from violating the invariants of the tree once constructed
//! while still allowing them to mutate parts of each element of the tree. The user can mutably traverse
//! the tree but the mutable references returns are hidden behind the `HalfPin<T>` type that forbids
//! mutating the aabbs.
//!
//! ### Unsafety
//!
//! Raw pointers are used for the container types in the container module
//! and for caching the results of finding colliding pairs.
//!
//! [`multi_rect`](Tree::multi_rect) uses unsafety to allow the user to have mutable references to elements
//! that belong to rectangle regions that don't intersect at the same time. This is why
//! the [`node::Aabb`] trait is unsafe.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/tiby312/broccoli/master/assets/logo.png",
    html_favicon_url = "https://raw.githubusercontent.com/tiby312/broccoli/master/assets/logo.png"
)]
//#![no_std]

#[cfg(doctest)]
mod test_readme {
    macro_rules! external_doc_test {
        ($x:expr) => {
            #[doc = $x]
            extern "C" {}
        };
    }

    external_doc_test!(include_str!("../README.md"));
}

#[macro_use]
extern crate alloc;

pub use axgeom;
pub use broccoli_tree::*;
pub use compt;

use alloc::vec::Vec;
use broccoli_tree::node::*;

//TODO use this thing!!!!
pub struct Accumulator<T> {
    expected_size: usize,
    inner: Vec<T>,
}
impl<T> Accumulator<T> {
    pub fn new(expected_size: usize) -> Accumulator<T> {
        Accumulator {
            expected_size,
            inner: Vec::with_capacity(expected_size),
        }
    }
    pub fn push(&mut self, a: T) {
        self.inner.push(a);
    }
    pub fn build(self) -> Vec<T> {
        assert_eq!(self.expected_size, self.inner.len());
        self.inner
    }
}

pub mod prelude {
    pub use super::queries::colfind::CollisionApi;
    pub use super::queries::knearest::KnearestApi;
    pub use super::queries::raycast::RaycastApi;
    pub use super::queries::rect::RectApi;
}
#[cfg(test)]
mod tests;

pub mod queries;

///Generic slice utility functions.
pub mod util;

//Implement for NotSortedTree, Tree, and NaiveSlice
trait QueriesHalfPin<T: Aabb> {
    
    /*
    /// # Examples
    ///
    /// ```
    /// use broccoli::{bbox,rect};
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
        let mut d = queries::draw::DrawClosure { line };

        queries::draw::draw(default_axis(), self.vistr(), &mut d, rect)
    }

    /// Find collisions between elements in this tree,
    /// with the specified slice of elements.
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::{bbox,rect};
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
        mut func: impl FnMut(HalfPin<&mut T>, HalfPin<&mut X>),
    ) {
        //TODO instead of create just a list of BBox, construct a tree using the dividers of the current tree.
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

        for i in other.iter_mut() {
            queries::rect::for_all_intersect_rect_mut(
                default_axis(),
                self.vistr_mut(),
                i,
                |r, a| func(a, HalfPin::new(*r)),
            );
        }
    }

    /// Find the closest `num` elements to the specified `point`.
    /// The user provides two functions:
    ///
    /// The result is returned as one `Vec`. The closest elements will
    /// appear first. Multiple elements can be returned
    /// with the same distance in the event of ties. These groups of elements are separated by
    /// one entry of `Option::None`. In order to iterate over each group,
    /// try using the slice function: `arr.split(|a| a.is_none())`
    ///
    /// # Examples
    ///
    ///```
    /// use broccoli::{bbox,rect};
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
    /// let buffer:Vec<_>=res.iter().map(|a|*a[0].bot.inner).collect();
    ///
    /// assert_eq!(buffer,vec![vec2(7,7),vec2(5,5)]);
    ///
    ///
    /// fn distance_squared(a:isize,b:isize)->isize{
    ///     let a=(a-b).abs();
    ///     a*a
    /// }
    ///```
    #[must_use]
    pub fn k_nearest_mut<'b, K: queries::knearest::Knearest<T>>(
        &'b mut self,
        point: Vec2<T::Num>,
        num: usize,
        ktrait: &mut K,
    ) -> queries::knearest::KResult<'b, T> {
        queries::knearest::knearest_mut(self, point, num, ktrait)
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
    /// use broccoli::{bbox,rect};
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
    pub fn raycast_mut<'b, R: queries::raycast::RayCast<T>>(
        &'b mut self,
        ray: axgeom::Ray<T::Num>,
        rtrait: &mut R,
    ) -> axgeom::CastResult<queries::raycast::CastAnswer<'b, T>> {
        queries::raycast::raycast_mut(self, ray, rtrait)
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::{bbox,rect};
    /// let mut bots = [bbox(rect(0,10,0,10),0u8)];
    /// let mut tree = broccoli::new(&mut bots);
    /// tree.for_all_intersect_rect_mut(&rect(9,20,9,20),|a|{
    ///    *a.unpack_inner()+=1;
    /// });
    ///
    /// assert_eq!(bots[0].inner,1);
    ///
    ///```
    pub fn for_all_intersect_rect_mut<'b, K: Aabb<Num = T::Num>>(
        &'b mut self,
        rect: K,
        func: impl FnMut(&mut K, HalfPin<&'b mut T>),
    ) {
        queries::rect::for_all_intersect_rect_mut(default_axis(), self.vistr_mut(), rect, func);
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::{bbox,rect};
    /// let mut bots = [rect(0,10,0,10),rect(20,30,20,30)];
    /// let mut tree = broccoli::new(&mut bots);
    /// let mut test = Vec::new();
    /// tree.for_all_in_rect(&rect(0,20,0,20),|a|{
    ///    test.push(a);
    /// });
    ///
    /// assert_eq!(test[0],&rect(0,10,0,10));
    ///
    pub fn for_all_in_rect_mut<'b, K: Aabb<Num = T::Num>>(
        &'b mut self,
        rect: K,
        func: impl FnMut(&mut K, HalfPin<&'b mut T>),
    ) {
        queries::rect::for_all_in_rect_mut(default_axis(), self.vistr_mut(), rect, func);
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::{bbox,rect};
    /// let mut bots = [bbox(rect(0,10,0,10),0u8)];
    /// let mut tree = broccoli::new(&mut bots);
    /// tree.for_all_not_in_rect_mut(&rect(10,20,10,20),|a|{
    ///    *a.unpack_inner()+=1;
    /// });
    ///
    /// assert_eq!(bots[0].inner,1);
    ///
    ///```
    pub fn for_all_not_in_rect_mut<'b, K: Aabb<Num = T::Num>>(
        &'b mut self,
        rect: K,
        mut func: impl FnMut(&mut K, HalfPin<&'b mut T>),
    ) {
        queries::rect::for_all_not_in_rect_mut(
            default_axis(),
            self.vistr_mut(),
            rect,
            move |r, a| (func)(r, a),
        );
    }
    */
}
