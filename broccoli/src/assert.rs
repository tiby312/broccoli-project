use axgeom::Axis;
use compt::Visitor;

use super::*;

///
/// Easily verifiable naive query algorithms.
///
pub struct Naive<'a, T> {
    pub(crate) inner: AabbPin<&'a mut [T]>,
}
impl<'a, T: Aabb> Naive<'a, T> {
    pub fn new(inner: &'a mut [T]) -> Self {
        Naive {
            inner: AabbPin::from_mut(inner),
        }
    }
    pub fn from_pinned(inner: AabbPin<&'a mut [T]>) -> Self {
        Naive { inner }
    }

    pub fn iter_mut(&mut self) -> AabbPinIter<T> {
        self.inner.borrow_mut().iter_mut()
    }
}

///
/// Compare query results between [`Tree`] and
/// the easily verifiable [`Naive`] versions.
///
pub struct Assert<'a, T> {
    pub(crate) inner: &'a mut [T],
}
impl<'a, T: Aabb> Assert<'a, T> {
    pub fn new(inner: &'a mut [T]) -> Self {
        Assert { inner }
    }
}

///panics if a broken broccoli tree invariant is detected.
///For debugging purposes only.
pub fn assert_tree_invariants<'a, T: Aabb>(tree: &Tree<'a, T>)
where
    T::Num: core::fmt::Debug,
{
    fn inner<A: Axis, T: Aabb>(axis: A, iter: compt::LevelIter<Vistr<Node<T, T::Num>>>)
    where
        T::Num: core::fmt::Debug,
    {
        fn a_bot_has_value<N: Num>(it: impl Iterator<Item = N>, val: N) -> bool {
            for b in it {
                if b == val {
                    return true;
                }
            }
            false
        }

        let ((_depth, nn), rest) = iter.next();
        let axis_next = axis.next();

        assert!(crate::queries::is_sorted_by(&nn.range, |a, b| a
            .get()
            .get_range(axis_next)
            .start
            .partial_cmp(&b.get().get_range(axis_next).start)));

        if let Some([start, end]) = rest {
            match nn.div {
                Some(div) => {
                    if nn.range.is_empty() {
                        assert_eq!(nn.cont.start, nn.cont.end);
                        let v: T::Num = Default::default();
                        assert_eq!(nn.cont.start, v);
                    } else {
                        let cont = &nn.cont;
                        for bot in nn.range.iter() {
                            assert!(bot.get().get_range(axis).contains(div));
                        }

                        assert!(a_bot_has_value(
                            nn.range.iter().map(|b| b.get().get_range(axis).start),
                            div
                        ));

                        for bot in nn.range.iter() {
                            assert!(cont.contains_range(bot.get().get_range(axis)));
                        }

                        assert!(a_bot_has_value(
                            nn.range.iter().map(|b| b.get().get_range(axis).start),
                            cont.start
                        ));
                        assert!(a_bot_has_value(
                            nn.range.iter().map(|b| b.get().get_range(axis).end),
                            cont.end
                        ));
                    }

                    inner(axis_next, start);
                    inner(axis_next, end);
                }
                None => {
                    for (_depth, n) in start.dfs_preorder_iter().chain(end.dfs_preorder_iter()) {
                        assert!(n.range.is_empty());
                        //assert!(n.cont.is_none());
                        assert_eq!(n.cont.start, nn.cont.end);
                        let v: T::Num = Default::default();
                        assert_eq!(n.cont.start, v);

                        assert!(n.div.is_none());
                    }
                }
            }
        }
    }

    inner(default_axis(), tree.vistr().with_depth(compt::Depth(0)))
}
