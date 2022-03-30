use super::*;

///A trait that gives the user callbacks at events in a recursive algorithm on the tree.
///The main motivation behind this trait was to track the time spent taken at each level of the tree
///during construction.
pub trait Splitter: Sized {
    ///Called to split this into two to be passed to the children.
    fn div(&mut self) -> (Self, Self);

    ///Called to add the results of the recursive calls on the children.
    fn add(&mut self, a: Self, b: Self);
}

pub fn recurse_seq_splitter<T: Aabb, S: NodeHandler, SS: Splitter>(
    vistr: CollVis<T, S>,
    mut splitter: SS,
    prevec: &mut PreVec,
    mut func: impl FnMut(PMut<T>, PMut<T>),
) -> SS {
    if let Some([left, right]) = vistr.collide_and_next(prevec, &mut func) {
        let (s1, s2) = splitter.div();
        let al = recurse_seq_splitter(left, s1, prevec, &mut func);
        let ar = recurse_seq_splitter(right, s2, prevec, &mut func);
        splitter.add(al, ar);
    }
    splitter
}
