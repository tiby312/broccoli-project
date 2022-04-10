//!
//! Find colliding pairs between two independant sets
//!

use super::{rect::RectApi, *};

pub fn intersect_with_tree_mut<'a, T: Aabb, X: Aabb<Num = T::Num> + 'a>(
    tree: &mut crate::tree::Tree<'a, T>,
    other: &mut crate::Tree<'a, X>,
    func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut X>),
) {
    intersect_with_iter_mut(tree, other.iter_mut(), func)
}

pub fn intersect_with_iter_mut<'a, 'x, T: Aabb, X: Aabb<Num = T::Num> + 'x>(
    tree: &mut crate::tree::Tree<'a, T>,
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
