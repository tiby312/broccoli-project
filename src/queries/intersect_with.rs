use super::{rect::RectApi, *};

pub trait IntersectWithApi<T: Aabb> {
    fn intersect_with_iter_mut<'a, X: Aabb<Num = T::Num> + 'a>(
        &mut self,
        other: impl Iterator<Item = AabbPin<&'a mut X>>,
        func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut X>),
    );

    fn intersect_with_tree_mut<'a, X: Aabb<Num = T::Num> + 'a>(
        &mut self,
        other: &mut crate::Tree<'a, X>,
        func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut X>),
    ) {
        self.intersect_with_iter_mut(other.iter_mut(), func)
    }
}

impl<'a, T: Aabb> IntersectWithApi<T> for crate::tree::Tree<'a, T> {
    fn intersect_with_iter_mut<'x, X: Aabb<Num = T::Num> + 'x>(
        &mut self,
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
            self.for_all_in_rect_mut(i, |r, a| func(a, r))
        }
    }
}
