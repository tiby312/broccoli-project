//!
//! Tree building blocks
//!

use super::*;

#[must_use]
pub struct NodeFinisher<'a, T: Aabb> {
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
    pub fn finish<S:Sorter<T>>(self,sorter:&mut S) -> Node<'a, T> {
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

        let cont = match self.axis {
            AxisDyn::X => {
                sorter.sort(axgeom::XAXIS.next(), self.mid);
                create_cont(axgeom::XAXIS, self.mid)
            }
            AxisDyn::Y => {
                sorter.sort(axgeom::YAXIS.next(), self.mid);
                create_cont(axgeom::YAXIS, self.mid)
            }
        };

        Node {
            num_elem: self.num_elem,
            range: AabbPin::new(self.mid),
            cont,
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

impl<'a, T: Aabb> TreeBuildVisitor<'a, T> {
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

            let rr = match self.axis {
                AxisDyn::X => construct_non_leaf(axgeom::XAXIS, self.bots),
                AxisDyn::Y => construct_non_leaf(axgeom::YAXIS, self.bots),
            };

            let finish_node = NodeFinisher {
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

impl splitter::Splitter for DefaultSorter{
    fn div(&mut self)->Self{
        DefaultSorter
    }
    fn add(&mut self,other:Self){

    }
}

#[derive(Copy, Clone, Default)]
pub struct NoSorter;

impl<T: Aabb> Sorter<T> for NoSorter {
    fn sort(&self, _axis: impl Axis, _bots: &mut [T]) {}
}

impl splitter::Splitter for NoSorter{
    fn div(&mut self)->Self{
        NoSorter
    }
    fn add(&mut self,other:Self){

    }
}
