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

///
/// height_seq_fallback: if a subtree has this height, it will be processed as one unit sequentially.
///
pub fn recurse_par_splitter<T: Aabb, N: NodeHandler, S: Splitter + Send>(
    vistr: CollVis<T, N>,
    prevec: &mut PreVec,
    height_seq_fallback: usize,
    mut func: impl FnMut(PMut<T>, PMut<T>) + Clone + Send,
    mut splitter: S,
) -> S
where
    T: Send,
    T::Num: Send,
{
    if vistr.vistr.get_height() <= height_seq_fallback {
        vistr.recurse_seq(prevec, &mut func);
    } else {
        let rest = vistr.collide_and_next(prevec, &mut func);
        let func2 = func.clone();
        if let Some([left, right]) = rest {
            let (s1, s2) = splitter.div();

            let (s1, s2) = rayon_core::join(
                || recurse_par_splitter(left, prevec, height_seq_fallback, func, s1),
                || {
                    let mut prevec = PreVec::new();
                    recurse_par_splitter(right, &mut prevec, height_seq_fallback, func2, s2)
                },
            );

            splitter.add(s1, s2);
        }
    }
    splitter
}
