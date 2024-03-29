//!
//! Tree building blocks
//!

use super::*;

///The default starting axis of a [`Tree`]. It is set to be the `X` axis.
///This means that the first divider is a 'vertical' line since it is
///partitioning space based off of the aabb's `X` value.
#[must_use]
pub const fn default_axis() -> axgeom::XAXIS {
    axgeom::XAXIS
}

///Expose a common Sorter trait so that we may have two version of the tree
///where one implementation actually does sort the tree, while the other one
///does nothing when sort() is called.
pub trait Sorter<T> {
    fn sort(&self, axis: impl Axis, bots: &mut [T]);
}

///Sorts the bots based on an axis.
#[inline(always)]
pub fn sweeper_update<I: Aabb, A: Axis>(axis: A, collision_botids: &mut [I]) {
    collision_botids.sort_unstable_by(|a, b| queries::cmp_aabb(axis, a, b));
}

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

            let Some(start)=middle[0..ml].iter().map(|a|a.range(axis).start).min_by(|a,b|{
                if a<b{
                    std::cmp::Ordering::Less
                }else{
                    std::cmp::Ordering::Greater
                }
            }) else{
                return Default::default()
            };

            let Some(end)=middle.iter().map(|a|a.range(axis).end).max_by(|a,b|{
                if a>b{
                    std::cmp::Ordering::Greater
                }else{
                    std::cmp::Ordering::Less
                }
            })else{
                return Default::default()
            };

            axgeom::Range { start, end }
            
            
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
            //num_elem: self.num_elem,
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

    #[deprecated(note = "Use TreeEmbryo")]
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
                    crate::queries::cmp_aabb(div_axis, a, b)
                });

                let med_val = med.range(div_axis).start;

                let (ml, ll) = partition_left(ll, |a| a.range(div_axis).end >= med_val);
                let (mr, rr) = partition_left(rr, |a| a.range(div_axis).start <= med_val);

                let ml_len = ml.len();
                let ll_len = ll.len();
                let rr_len = rr.len();
                let mr_len = mr.len();

                //At this point we have:
                // [ml,ll,mr,rr]
                //move stuff around so we have:
                // [ml,mr,ll,rr]
                {
                    let (_, rest) = bots.split_at_mut(ml_len);
                    let (arr, _) = rest.split_at_mut(ll_len + 1 + mr_len);
                    swap_slice_different_sizes(arr, ll_len)
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

    #[deprecated(note = "Use TreeEmbryo")]
    pub fn recurse_seq<S: Sorter<T>>(self, sorter: &mut S, buffer: &mut Vec<Node<'a, T, T::Num>>) {
        let NodeBuildResult { node, rest } = self.build_and_next();
        buffer.push(node.finish(sorter));
        if let Some([left, right]) = rest {
            left.recurse_seq(sorter, buffer);
            right.recurse_seq(sorter, buffer);
        }
    }
}

pub struct TreeEmbryo<'a, T, N> {
    total_num_nodes: usize,
    target_num_nodes: usize,
    nodes: Vec<Node<'a, T, N>>,
}
impl<'a, T: Aabb> TreeEmbryo<'a, T, T::Num> {
    pub fn new(bots: &'a mut [T]) -> (TreeEmbryo<'a, T, T::Num>, TreeBuildVisitor<'a, T>) {
        let num_level = num_level::default(bots.len());
        Self::with_num_level(bots, num_level)
    }
    pub fn with_num_level(
        bots: &'a mut [T],
        num_levels: usize,
    ) -> (TreeEmbryo<'a, T, T::Num>, TreeBuildVisitor<'a, T>) {
        assert!(num_levels >= 1);
        let v = TreeBuildVisitor {
            bots,
            current_height: num_levels - 1,
            axis: default_axis().to_dyn(),
        };

        //Minus 1 because the embryo might be split.
        //all we know is that we will use this embryo for at least half of the
        //current level.
        let num_nodes = num_level::num_nodes(num_levels);
        let nodes = Vec::with_capacity(num_nodes / 2);
        (
            TreeEmbryo {
                total_num_nodes: num_nodes,
                nodes,
                target_num_nodes: num_nodes,
            },
            v,
        )
    }
    pub fn add(&mut self, node: Node<'a, T, T::Num>) {
        self.nodes.push(node);
    }

    pub fn div(&mut self) -> TreeEmbryo<'a, T, T::Num> {
        self.target_num_nodes /= 2;

        TreeEmbryo {
            total_num_nodes: self.total_num_nodes,
            target_num_nodes: self.target_num_nodes,
            nodes: Vec::with_capacity(self.target_num_nodes / 2),
        }
    }
    pub fn combine(&mut self, a: Self) -> &mut Self {
        //assert_eq!(self.target_num_nodes, self.nodes.len());
        //assert_eq!(a.target_num_nodes, a.nodes.len());
        //assert_eq!(self.nodes.len(),a.nodes.len());

        self.target_num_nodes *= 2;
        self.nodes.extend(a.nodes);
        self
    }
    pub fn finish(self) -> Tree<'a, T> {
        assert_eq!(self.total_num_nodes, self.nodes.len());
        Tree::from_nodes(self.nodes)
    }

    pub fn into_nodes(self) -> Vec<Node<'a, T, T::Num>> {
        assert_eq!(self.total_num_nodes, self.nodes.len());
        self.nodes
    }

    /// Recuse sequentially
    pub fn recurse<S: Sorter<T>>(&mut self, a: TreeBuildVisitor<'a, T>, sorter: &mut S)
    where
        T: ManySwap,
    {
        let NodeBuildResult { node, rest } = a.build_and_next();
        self.add(node.finish(sorter));
        if let Some([left, right]) = rest {
            self.recurse(left, sorter);
            self.recurse(right, sorter);
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct DefaultSorter;

impl<T: Aabb> Sorter<T> for DefaultSorter {
    fn sort(&self, axis: impl Axis, bots: &mut [T]) {
        crate::build::sweeper_update(axis, bots);
    }
}

fn partition_left<T>(arr: &mut [T], mut func: impl FnMut(&T) -> bool) -> (&mut [T], &mut [T]) {
    let mut m = 0;
    for a in 0..arr.len() {
        if func(&arr[a]) {
            arr.swap(a, m);
            m += 1;
        }
    }
    arr.split_at_mut(m)
}

// swap a (l)(s) to (s)(l)
// only enough elements of l are moved to swap s in.
fn swap_slice_different_sizes<T>(arr: &mut [T], l_len: usize) {
    let s_len = arr.len() - l_len;
    let copy_length = s_len.min(l_len);

    let (rest, src) = arr.split_at_mut(arr.len() - copy_length);
    let (target, _) = rest.split_at_mut(copy_length);
    src.swap_with_slice(target);
}
