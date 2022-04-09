use super::*;

use crate::tree::splitter::Splitter;

pub fn colliding_pairs_splitter<'a, T: Aabb + 'a, SO: NodeHandler, SS: Splitter>(
    bu: &mut impl CollidingPairsBuilder<'a, T, SO>,
    splitter: SS,
    mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
) -> SS {
    pub fn recurse_seq_splitter<T: Aabb, S: NodeHandler, SS: Splitter>(
        vistr: CollVis<T, S>,
        splitter: SS,
        prevec: &mut PreVec,
        func: &mut impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
    ) -> SS {
        let (n, rest) = vistr.collide_and_next(prevec, func);

        if let Some([left, right]) = rest {
            let (s1, s2) = splitter.div();
            n.finish();
            let al = recurse_seq_splitter(left, s1, prevec, func);
            let ar = recurse_seq_splitter(right, s2, prevec, func);
            al.add(ar)
        } else {
            splitter
        }
    }
    let mut prevec = PreVec::new();
    recurse_seq_splitter(
        bu.colliding_pairs_builder(),
        splitter,
        &mut prevec,
        &mut func,
    )
}

#[cfg(feature = "rayon")]
pub fn colliding_pairs_splitter_par<'a, T: Aabb + 'a, SO: NodeHandler, SS: Splitter + Send>(
    bu: &mut impl CollidingPairsBuilder<'a, T, SO>,
    func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
    splitter: SS,
) -> SS
where
    T: Send,
    T::Num: Send,
{
    ///
    /// height_seq_fallback: if a subtree has this height, it will be processed as one unit sequentially.
    ///
    pub fn recurse_par_splitter<T: Aabb, N: NodeHandler, S: Splitter + Send>(
        vistr: CollVis<T, N>,
        prevec: &mut PreVec,
        height_seq_fallback: usize,
        mut func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>) + Clone + Send,
        splitter: S,
    ) -> S
    where
        T: Send,
        T::Num: Send,
    {
        if vistr.get_height() <= height_seq_fallback {
            vistr.recurse_seq(prevec, &mut func);
            splitter
        } else {
            let func2 = func.clone();
            let (n, rest) = vistr.collide_and_next(prevec, &mut func);
            if let Some([left, right]) = rest {
                let (s1, s2) = splitter.div();

                let (s1, s2) = rayon::join(
                    || {
                        let (prevec, func) = n.finish();
                        recurse_par_splitter(left, prevec, height_seq_fallback, func.clone(), s1)
                    },
                    || {
                        let mut prevec = PreVec::new();
                        recurse_par_splitter(right, &mut prevec, height_seq_fallback, func2, s2)
                    },
                );

                s1.add(s2)
            } else {
                splitter
            }
        }
    }
    let mut prevec = PreVec::new();
    let h = bu.height_seq_fallback();
    recurse_par_splitter(bu.colliding_pairs_builder(), &mut prevec, h, func, splitter)
}
