use super::*;

///
/// The main primitive to build a tree.
///
pub struct TreeBuildVisitor<'a, T, S> {
    bots: &'a mut [T],
    current_height: usize,
    sorter: S,
    is_xaxis: bool,
}

pub struct NodeBuildResult<'a, T: Aabb, S> {
    pub node: NodeFinisher<'a, T, S>,
    pub rest: Option<[TreeBuildVisitor<'a, T, S>; 2]>,
}

impl<'a, T: Aabb, S: Sorter> TreeBuildVisitor<'a, T, S> {
    pub fn new(num_levels: usize, bots: &'a mut [T], sorter: S) -> TreeBuildVisitor<'a, T, S> {
        assert!(num_levels >= 1);
        TreeBuildVisitor {
            bots,
            current_height: num_levels - 1,
            sorter,
            is_xaxis: true,
        }
    }

    pub fn get_height(&self) -> usize {
        self.current_height
    }
    pub fn build_and_next(self) -> NodeBuildResult<'a, T, S> {
        //leaf case
        if self.current_height == 0 {
            let node = NodeFinisher {
                mid: self.bots,
                div: None,
                is_xaxis: self.is_xaxis,
                sorter: self.sorter,
            };

            NodeBuildResult { node, rest: None }
        } else {
            fn construct_non_leaf<T: Aabb>(
                div_axis: impl Axis,
                bots: &mut [T],
            ) -> ConstructResult<T> {
                if bots.is_empty() {
                    return ConstructResult {
                        mid: &mut [],
                        div: None,
                        left: &mut [],
                        right: &mut [],
                    };
                }

                let med_index = bots.len() / 2;
                let (_, med, _) = bots.select_nth_unstable_by(med_index, move |a, b| {
                    crate::util::compare_bots(div_axis, a, b)
                });

                let med_val = med.get().get_range(div_axis).start;

                //It is very important that the median bot end up be binned into the middile bin.
                //We know this must be true because we chose the divider to be the medians left border,
                //and we binned so that all bots who intersect with the divider end up in the middle bin.
                //Very important that if a bots border is exactly on the divider, it is put in the middle.
                //If this were not true, there is no guarantee that the middile bin has bots in it even
                //though we did pick a divider.
                let binned = oned::bin_middle_left_right(div_axis, &med_val, bots);

                ConstructResult {
                    mid: binned.middle,
                    div: Some(med_val),
                    left: binned.left,
                    right: binned.right,
                }
            }

            let rr = if self.is_xaxis {
                construct_non_leaf(axgeom::XAXIS, self.bots)
            } else {
                construct_non_leaf(axgeom::YAXIS, self.bots)
            };

            let finish_node = NodeFinisher {
                mid: rr.mid,
                div: rr.div,
                is_xaxis: self.is_xaxis,
                sorter: self.sorter,
            };

            let left = rr.left;
            let right = rr.right;

            NodeBuildResult {
                node: finish_node,
                rest: Some([
                    TreeBuildVisitor {
                        bots: left,
                        current_height: self.current_height.saturating_sub(1),
                        sorter: self.sorter,
                        is_xaxis: !self.is_xaxis,
                    },
                    TreeBuildVisitor {
                        bots: right,
                        current_height: self.current_height.saturating_sub(1),
                        sorter: self.sorter,
                        is_xaxis: !self.is_xaxis,
                    },
                ]),
            }
        }
    }

    pub fn recurse_seq(self, res: &mut Vec<Node<'a, T>>) {
        let NodeBuildResult { node, rest } = self.build_and_next();
        res.push(node.finish());
        if let Some([left, right]) = rest {
            left.recurse_seq(res);
            right.recurse_seq(res);
        }
    }
}

struct ConstructResult<'a, T: Aabb> {
    div: Option<T::Num>,
    mid: &'a mut [T],
    right: &'a mut [T],
    left: &'a mut [T],
}
