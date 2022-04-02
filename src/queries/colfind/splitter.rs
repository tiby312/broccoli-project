use super::*;

use crate::tree::splitter::Splitter;


pub fn recurse_seq_splitter<T: Aabb, S: NodeHandler, SS: Splitter>(
    vistr: CollVis<T, S>,
    mut splitter: SS,
    prevec: &mut PreVec,
    mut func: impl FnMut(HalfPin<&mut T>, HalfPin<&mut T>),
) -> SS {
    let (n, rest) = vistr.collide_and_next(prevec, &mut func);

    if let Some([left, right]) = rest {
        let (s1, s2) = splitter.div();
        n.finish();
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
    mut func: impl FnMut(HalfPin<&mut T>, HalfPin<&mut T>) + Clone + Send,
    mut splitter: S,
) -> S
where
    T: Send,
    T::Num: Send,
{
    if vistr.vistr.get_height() <= height_seq_fallback {
        vistr.recurse_seq(prevec, &mut func);
    } else {
        let func2 = func.clone();
        let (n, rest) = vistr.collide_and_next(prevec, func);
        if let Some([left, right]) = rest {
            let (s1, s2) = splitter.div();

            let (s1, s2) = rayon_core::join(
                || {
                    let (prevec, func) = n.finish();
                    recurse_par_splitter(left, prevec, height_seq_fallback, func, s1)
                },
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
