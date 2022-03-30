use super::*;

///
/// height_seq_fallback: if a subtree has this height, it will be processed as one unit sequentially.
///
pub fn recurse_par<T: Aabb, N: NodeHandler>(
    vistr: CollVis<T, N>,
    prevec: &mut PreVec,
    height_seq_fallback: usize,
    mut func: impl FnMut(PMut<T>, PMut<T>) + Clone + Send + Sync,
) where
    T: Send + Sync,
    T::Num: Send + Sync,
{
    if vistr.vistr.get_height() <= height_seq_fallback {
        vistr.recurse_seq(prevec, func);
    } else {
        let rest = vistr.collide_and_next(prevec, &mut func);
        let func2 = func.clone();
        if let Some([left, right]) = rest {
            rayon_core::join(
                || {
                    recurse_par(left, prevec, height_seq_fallback, func);
                },
                || {
                    let mut prevec = PreVec::new();
                    recurse_par(right, &mut prevec, height_seq_fallback, func2);
                },
            );
        }
    }
}
