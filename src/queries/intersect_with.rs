use super::{rect::RectApi, *};

pub trait IntersectWithApi<T: Aabb> {
    fn intersect_with_mut<X: Aabb<Num = T::Num>>(
        &mut self,
        other: &mut [X],
        func: impl FnMut(HalfPin<&mut T>, HalfPin<&mut X>),
    );
}

impl<'a, T: Aabb> IntersectWithApi<T> for crate::tree::Tree<'a, T> {
    fn intersect_with_mut<X: Aabb<Num = T::Num>>(
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
            self.for_all_in_rect_mut(i, |r, a| func(a, HalfPin::new(r)))
        }
    }
}
