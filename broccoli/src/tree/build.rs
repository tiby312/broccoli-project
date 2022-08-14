//!
//! Tree building blocks
//!

use super::*;

#[must_use]
pub struct NodeFinisher<'a, T: Aabb> {
    axis: AxisDyn,
    div: Option<T::Num>, //This can be null if there are no bots left at all
    mid: &'a mut [T],
    middle_left_len: Option<usize>,
    min_elem: usize,
    num_elem: usize,
}
impl<'a, T: Aabb> NodeFinisher<'a, T> {
    pub fn get_min_elem(&self) -> usize {
        self.min_elem
    }

    pub fn get_num_elem(&self) -> usize {
        self.num_elem
    }
    #[inline(always)]
    #[must_use]
    pub fn finish<S: Sorter<T>>(self, sorter: &mut S) -> Node<'a, T, T::Num> {
        fn create_cont<A: Axis, T: Aabb>(
            axis: A,
            middle: &[T],
            ml: Option<usize>,
        ) -> axgeom::Range<T::Num> {
            let ml = if let Some(ml) = ml { ml } else { middle.len() };

            match middle.split_first() {
                Some((first, rest)) => {
                    let start = {
                        let mut min = first.get().get_range(axis).start;

                        for a in rest[0..ml - 1].iter() {
                            let start = a.get().get_range(axis).start;

                            if start < min {
                                min = start;
                            }
                        }
                        min
                    };

                    let end = {
                        let mut max = first.get().get_range(axis).end;

                        //The bots to the right of the divier
                        //are more likely to  contain the max
                        //rightmost aabb edge.
                        for a in rest.iter().rev() {
                            let k = a.get().get_range(axis).end;

                            if k > max {
                                max = k;
                            }
                        }
                        max
                    };

                    axgeom::Range { start, end }
                }
                None => axgeom::Range {
                    start: Default::default(),
                    end: Default::default(),
                },
            }
        }

        let cont = match self.axis {
            AxisDyn::X => {
                //Create cont first otherwise middle_left_len is no longer valid.
                let c = create_cont(axgeom::XAXIS, self.mid, self.middle_left_len);
                sorter.sort(axgeom::XAXIS.next(), self.mid);
                c
            }
            AxisDyn::Y => {
                let c = create_cont(axgeom::YAXIS, self.mid, self.middle_left_len);
                sorter.sort(axgeom::YAXIS.next(), self.mid);
                c
            }
        };

        Node {
            cont,
            min_elem: self.min_elem,
            num_elem: self.num_elem,
            range: AabbPin::new(self.mid),
            div: self.div,
        }
    }
}

///
/// The main primitive to build a tree.
///
pub struct TreeBuildVisitor<'a, T> {
    bots: &'a mut [T],
    current_height: usize,
    axis: AxisDyn,
}

pub struct NodeBuildResult<'a, T: Aabb> {
    pub node: NodeFinisher<'a, T>,
    pub rest: Option<[TreeBuildVisitor<'a, T>; 2]>,
}

impl<'a, T: Aabb + ManySwap> TreeBuildVisitor<'a, T> {
    pub fn get_bots(&self) -> &[T] {
        self.bots
    }
    #[must_use]
    pub fn new(num_levels: usize, bots: &'a mut [T]) -> TreeBuildVisitor<'a, T> {
        assert!(num_levels >= 1);
        TreeBuildVisitor {
            bots,
            current_height: num_levels - 1,
            axis: default_axis().to_dyn(),
        }
    }
    #[must_use]
    pub fn get_height(&self) -> usize {
        self.current_height
    }
    #[must_use]
    pub fn build_and_next(self) -> NodeBuildResult<'a, T> {
        //leaf case
        if self.current_height == 0 {
            let node = NodeFinisher {
                middle_left_len: None,
                mid: self.bots,
                div: None,
                axis: self.axis,
                min_elem: 0,
                num_elem: 0,
            };

            NodeBuildResult { node, rest: None }
        } else {
            fn construct_non_leaf<T: Aabb>(
                div_axis: impl Axis,
                bots: &mut [T],
            ) -> (NodeFinisher<T>, &mut [T], &mut [T]) {
                if bots.is_empty() {
                    return (
                        NodeFinisher {
                            middle_left_len: None,
                            mid: bots,
                            div: None,
                            axis: div_axis.to_dyn(),
                            min_elem: 0,
                            num_elem: 0,
                        },
                        &mut [],
                        &mut [],
                    );
                }

                let med_index = bots.len() / 2;

                let (ll, med, rr) = bots.select_nth_unstable_by(med_index, move |a, b| {
                    crate::util::compare_bots(div_axis, a, b)
                });

                let med_val = med.get().get_range(div_axis).start;

                let (ml, ll) = {
                    let mut m = 0;
                    for a in 0..ll.len() {
                        if ll[a].get().get_range(div_axis).end >= med_val {
                            //keep
                            ll.swap(a, m);
                            m += 1;
                        }
                    }
                    ll.split_at_mut(m)
                };

                let (mr, rr) = {
                    let mut m = 0;
                    for a in 0..rr.len() {
                        if rr[a].get().get_range(div_axis).start <= med_val {
                            //keep
                            rr.swap(a, m);
                            m += 1;
                        }
                    }
                    rr.split_at_mut(m)
                };

                let ml_len = ml.len();
                let ll_len = ll.len();
                let rr_len = rr.len();

                let mr_len = mr.len();

                {
                    let (_, rest) = bots.split_at_mut(ml_len);
                    let (ll, rest) = rest.split_at_mut(ll_len);
                    let (mr2, _) = rest.split_at_mut(1 + mr_len);

                    let (a, b) = if mr2.len() < ll.len() {
                        let (a, _) = ll.split_at_mut(mr2.len());
                        (a, mr2)
                    } else {
                        let (_, a) = mr2.split_at_mut(mr2.len() - ll.len());
                        (a, ll)
                    };
                    a.swap_with_slice(b);
                }

                let left_len = ll_len;
                let right_len = rr_len;
                let mid_len = ml_len + 1 + mr_len;

                let middle_left_len = Some(ml_len + 1);

                let (mid, rest) = bots.split_at_mut(mid_len);
                let (left, right) = rest.split_at_mut(left_len);

                (
                    NodeFinisher {
                        middle_left_len,
                        mid,
                        div: Some(med_val),
                        axis: div_axis.to_dyn(),
                        min_elem: left_len.min(right_len),
                        num_elem: left_len + right_len,
                    },
                    left,
                    right,
                )
            }

            let (finish_node, left, right) = match self.axis {
                AxisDyn::X => construct_non_leaf(axgeom::XAXIS, self.bots),
                AxisDyn::Y => construct_non_leaf(axgeom::YAXIS, self.bots),
            };

            NodeBuildResult {
                node: finish_node,
                rest: Some([
                    TreeBuildVisitor {
                        bots: left,
                        current_height: self.current_height.saturating_sub(1),
                        axis: self.axis.next(),
                    },
                    TreeBuildVisitor {
                        bots: right,
                        current_height: self.current_height.saturating_sub(1),
                        axis: self.axis.next(),
                    },
                ]),
            }
        }
    }

    pub fn recurse_seq<S: Sorter<T>>(self, sorter: &mut S, buffer: &mut Vec<Node<'a, T, T::Num>>) {
        let NodeBuildResult { node, rest } = self.build_and_next();
        buffer.push(node.finish(sorter));
        if let Some([left, right]) = rest {
            left.recurse_seq(sorter, buffer);
            right.recurse_seq(sorter, buffer);
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct DefaultSorter;

impl<T: Aabb> Sorter<T> for DefaultSorter {
    fn sort(&self, axis: impl Axis, bots: &mut [T]) {
        crate::util::sweeper_update(axis, bots);
    }
}

impl splitter::Splitter for DefaultSorter {
    fn div(&mut self) -> Self {
        DefaultSorter
    }
    fn add(&mut self, _other: Self) {}
}

#[derive(Copy, Clone, Default)]
pub struct NoSorter;

impl<T: Aabb> Sorter<T> for NoSorter {
    fn sort(&self, _axis: impl Axis, _bots: &mut [T]) {}
}

impl splitter::Splitter for NoSorter {
    fn div(&mut self) -> Self {
        NoSorter
    }
    fn add(&mut self, _other: Self) {}
}
