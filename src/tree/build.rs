//!
//! Tree building blocks
//!

use super::*;

#[must_use]
pub struct NodeFinisher<'a, T: Aabb> {
    cont: [T::Num; 2],
    axis: AxisDyn,
    div: Option<T::Num>, //This can be null if there are no bots left at all
    mid: &'a mut [T],
    num_elem: usize,
}
impl<'a, T: Aabb> NodeFinisher<'a, T> {
    pub fn get_num_elem(&self) -> usize {
        self.num_elem
    }
    #[inline(always)]
    #[must_use]
    pub fn finish<S: Sorter<T>>(self, sorter: &mut S) -> Node<'a, T> {
        match self.axis {
            AxisDyn::X => {
                sorter.sort(axgeom::XAXIS.next(), self.mid);
            }
            AxisDyn::Y => {
                sorter.sort(axgeom::YAXIS.next(), self.mid);
            }
        };

        let cont = Range {
            start: self.cont[0],
            end: self.cont[1],
        };

        Node {
            num_elem: self.num_elem,
            range: AabbPin::new(self.mid),
            cont,
            div: self.div,
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

impl<'a, T: Aabb + ManySwappable> TreeBuildVisitor<'a, T> {
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
            let cont = match self.axis {
                AxisDyn::X => create_cont(axgeom::XAXIS, self.bots),
                AxisDyn::Y => create_cont(axgeom::YAXIS, self.bots),
            };

            let node = NodeFinisher {
                cont: [cont.start, cont.end],
                mid: self.bots,
                div: None,
                axis: self.axis,
                num_elem: 0,
            };

            NodeBuildResult { node, rest: None }
        } else {
            fn construct_non_leaf<T: Aabb>(
                div_axis: impl Axis,
                bots: &mut [T],
            ) -> ConstructResult<T> {
                if bots.is_empty() {
                    return ConstructResult {
                        cont: [std::default::Default::default(); 2],
                        mid: &mut [],
                        div: None,
                        left: &mut [],
                        right: &mut [],
                    };
                }

                let med_index = bots.len() / 2;

                let (min_cont, med_val, left_len, mid_len) = {
                    let (ll, med, rr) = bots.select_nth_unstable_by(med_index, move |a, b| {
                        crate::util::compare_bots(div_axis, a, b)
                    });

                    let med_val = med.get().get_range(div_axis).start;

                    fn bin_left<A: Axis, T: Aabb>(
                        axis: A,
                        bots: &mut [T],
                        bound: T::Num,
                    ) -> (&mut [T], &mut [T]) {
                        let mut m = bots.len();
                        for a in (0..bots.len()).rev() {
                            if bots[a].get().get_range(axis).end >= bound {
                                //keep
                                m -= 1;
                                bots.swap(a, m);
                            }
                        }
                        bots.split_at_mut(m)
                    }
                    fn bin_right<A: Axis, T: Aabb>(
                        axis: A,
                        bots: &mut [T],
                        bound: T::Num,
                    ) -> (&mut [T], &mut [T]) {
                        let mut m = 0;
                        for a in 0..bots.len() {
                            if bots[a].get().get_range(axis).start <= bound {
                                //keep
                                bots.swap(a, m);
                                m += 1;
                            }
                        }
                        bots.split_at_mut(m)
                    }

                    let (ll, ml) = bin_left(div_axis, ll, med_val);
                    let (mr, _) = bin_right(div_axis, rr, med_val);

                    let min_cont = {
                        let mut ret = med_val;
                        for a in ml.iter() {
                            let k = a.get().get_range(div_axis).start;
                            if k < ret {
                                ret = k;
                            }
                        }
                        ret
                    };

                    (min_cont, med_val, ll.len(), ml.len() + 1 + mr.len())
                };

                // Re-arrange so we have preorder to match the nodes being
                // in pre-rder.
                let (a, b) = if left_len > mid_len {
                    // |------left-----|--mid--|---right--|
                    let (a, rest) = bots.split_at_mut(mid_len);
                    let (_, rest) = rest.split_at_mut(left_len - mid_len);
                    let (b, _) = rest.split_at_mut(mid_len);
                    (a, b)
                } else {
                    // |-left-|------mid------|---right--|
                    let (a, rest) = bots.split_at_mut(left_len);
                    let (_, rest) = rest.split_at_mut(mid_len - left_len);
                    let (b, _) = rest.split_at_mut(left_len);
                    (a, b)
                };
                a.swap_with_slice(b);
                let (middle, rest) = bots.split_at_mut(mid_len);
                let (left, right) = rest.split_at_mut(left_len);

                // If we want in-order use this instead of the above
                //let (left, rest) = bots.split_at_mut(left_len);
                //let (middle, right) = rest.split_at_mut(mid_len);

                let max_cont = {
                    let mut ret = med_val;
                    for a in middle.iter() {
                        let k = a.get().get_range(div_axis).end;
                        if k > ret {
                            ret = k;
                        }
                    }
                    ret
                };

                ConstructResult {
                    cont: [min_cont, max_cont],
                    mid: middle,
                    div: Some(med_val),
                    left,
                    right,
                }
            }

            let rr = match self.axis {
                AxisDyn::X => construct_non_leaf(axgeom::XAXIS, self.bots),
                AxisDyn::Y => construct_non_leaf(axgeom::YAXIS, self.bots),
            };

            let finish_node = NodeFinisher {
                cont: rr.cont,
                mid: rr.mid,
                div: rr.div,
                axis: self.axis,
                num_elem: rr.left.len().min(rr.right.len()),
            };

            let left = rr.left;
            let right = rr.right;

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
}

struct ConstructResult<'a, T: Aabb> {
    cont: [T::Num; 2],
    div: Option<T::Num>,
    mid: &'a mut [T],
    right: &'a mut [T],
    left: &'a mut [T],
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
