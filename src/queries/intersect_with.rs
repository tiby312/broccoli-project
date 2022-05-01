//!
//! Find colliding pairs between two independent sets
//!

use super::{rect::RectApi, *};

impl<'a, T: Aabb> Tree2<'a, T> {
    pub fn find_colliding_pairs_with<X: Aabb<Num = T::Num>>(
        &mut self,
        other: &mut crate::Tree2<X>,
        func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut X>),
    ) {
        queries::intersect_with::intersect_with_tree_mut(&mut self.inner, &mut other.inner, func)
    }

    pub fn find_colliding_pairs_with_iter<'x, X: Aabb<Num = T::Num> + 'x>(
        &mut self,
        other: impl Iterator<Item = AabbPin<&'x mut X>>,
        func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut X>),
    ) {
        queries::intersect_with::intersect_with_iter_mut(&mut self.inner, other, func)
    }
}

pub fn intersect_with_tree_mut<T: Aabb, X: Aabb<Num = T::Num>>(
    tree: &mut crate::tree::Tree<T>,
    other: &mut crate::Tree<X>,
    func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut X>),
) {
    intersect_with_iter_mut(tree, other.iter_mut(), func)
}

pub fn intersect_with_iter_mut<'x, T: Aabb, X: Aabb<Num = T::Num> + 'x>(
    tree: &mut crate::tree::Tree<T>,
    other: impl Iterator<Item = AabbPin<&'x mut X>>,
    mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut X>),
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

    for i in other {
        tree.for_all_in_rect_mut(i, |r, a| func(a, r))
    }
}
