use super::*;

struct NodeFinisher<'a, T: Aabb, S> {
    is_xaxis: bool,
    div: Option<T::Num>, //This can be null if there are no bots left at all
    mid: &'a mut [T],
    sorter: S,
}
impl<'a, T: Aabb, S: Sorter> NodeFinisher<'a, T, S> {
    #[inline(always)]
    fn finish(self) -> Node<'a, T> {
        let cont = if self.is_xaxis {
            self.sorter.sort(axgeom::XAXIS.next(), self.mid);
            create_cont(axgeom::XAXIS, self.mid)
        } else {
            self.sorter.sort(axgeom::YAXIS.next(), self.mid);
            create_cont(axgeom::YAXIS, self.mid)
        };

        Node {
            range: PMut::new(self.mid),
            cont,
            div: self.div,
        }
    }
}

pub fn start_build<T: Aabb>(height: usize, bots: &mut [T]) -> TreeBister<T, DefaultSorter> {
    TreeBister {
        bots,
        current_height: height,
        sorter: DefaultSorter,
        is_xaxis: true,
    }
}

pub struct TreeBister<'a, T, S> {
    bots: &'a mut [T],
    current_height: usize,
    sorter: S,
    is_xaxis: bool,
}

impl<'a, T: Aabb, S: Sorter> TreeBister<'a, T, S> {
    fn build_and_next(self) -> (NodeFinisher<'a, T, S>, Option<[TreeBister<'a, T, S>; 2]>) {
        //leaf case
        if self.current_height == 1 {
            let n = NodeFinisher {
                mid: self.bots,
                div: None,
                is_xaxis: self.is_xaxis,
                sorter: self.sorter,
            };

            (n, None)
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

            (
                finish_node,
                Some([
                    TreeBister {
                        bots: left,
                        current_height: self.current_height.saturating_sub(1),
                        sorter: self.sorter,
                        is_xaxis: !self.is_xaxis,
                    },
                    TreeBister {
                        bots: right,
                        current_height: self.current_height.saturating_sub(1),
                        sorter: self.sorter,
                        is_xaxis: !self.is_xaxis,
                    },
                ]),
            )
        }
    }

    fn recurse_seq(self, res: &mut Vec<Node<'a, T>>) {
        let mut stack = vec![];
        stack.push(self);

        while let Some(s) = stack.pop() {
            let (n, rest) = s.build_and_next();
            res.push(n.finish());
            if let Some([left, right]) = rest {
                stack.push(left);
                stack.push(right);
            }
        }
    }
}

fn create_cont<A: Axis, T: Aabb>(axis: A, middle: &[T]) -> axgeom::Range<T::Num> {
    match middle.split_first() {
        Some((first, rest)) => {
            let mut min = first.get().get_range(axis).start;
            let mut max = first.get().get_range(axis).end;

            for a in rest.iter() {
                let start = &a.get().get_range(axis).start;
                let end = &a.get().get_range(axis).end;

                if *start < min {
                    min = *start;
                }

                if *end > max {
                    max = *end;
                }
            }

            axgeom::Range {
                start: min,
                end: max,
            }
        }
        None => axgeom::Range {
            start: Default::default(),
            end: Default::default(),
        },
    }
}

struct ConstructResult<'a, T: Aabb> {
    div: Option<T::Num>,
    mid: &'a mut [T],
    right: &'a mut [T],
    left: &'a mut [T],
}
