use crate::query::inner_prelude::*;
use crate::query::rect::*;

///Find all intersecting pairs between the elements in this dinotree, and the specified elements.
///No intersecting pairs within each group are looked for, only those between the two groups.
///For best performance the group that this tree is built around should be the bigger of the two groups.
///Since the dividers of the tree are used to divide and conquer the problem.
///If the other group is bigger, consider building the DinoTree around that group instead, and
///leave this group has a list of bots.
///
///Currently this is implemented naively using for_all_intersect_rect_mut().
///But using the api, it is possible to build up a tree using the current trees dividers
///to exploit the divide and conquer properties of this problem.
///The two trees could be recursed at the same time to break up the problem.
pub fn intersect_with_mut<A: Axis,N:Node, X: Aabb<Num = N::Num>>(
    axis:A,
    mut vistr:VistrMut<N>,
    b: &mut [X],
    func: impl Fn(PMut<N::T>, PMut<X>),
) {
    //TODO instead of create just a list of BBox, construct a tree using the dividors of the current tree.
    //This way we can paralleliz this function.

    for mut i in PMut::new(b).iter_mut() {
        let rect = *i.get();
        
        for_all_intersect_rect_mut(axis,vistr.create_wrap_mut(), &rect, |a| {
            func(a, i.as_mut());
        });
        
    }
}
