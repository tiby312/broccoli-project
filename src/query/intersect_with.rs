//! Find collisions between two groups
//
use crate::query::inner_prelude::*;
use crate::query::rect::*;



use super::Queries;

///Intersect functions that can be called on a tree.
pub trait IntersectQuery<'a>: Queries<'a>+RectQuery<'a>{

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
            self.for_all_intersect_rect_mut( &rect, |a| {
                func(a, i.borrow_mut());
            });
        }
    }


}

