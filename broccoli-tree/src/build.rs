//!
//! Tree building blocks
//!

use super::*;

#[must_use]
pub struct NodeFinisher<'a, T: Aabb, S> {
    is_xaxis: bool,
    div: Option<T::Num>, //This can be null if there are no bots left at all
    mid: &'a mut [T],
    sorter: S,
}
impl<'a, T: Aabb, S: Sorter> NodeFinisher<'a, T, S> {
    #[inline(always)]
    #[must_use]
    pub fn finish(self) -> Node<'a, T> {
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

        let cont = if self.is_xaxis {
            self.sorter.sort(axgeom::XAXIS.next(), self.mid);
            create_cont(axgeom::XAXIS, self.mid)
        } else {
            self.sorter.sort(axgeom::YAXIS.next(), self.mid);
            create_cont(axgeom::YAXIS, self.mid)
        };

        Node {
            range: AabbPin::new(self.mid),
            cont,
            div: self.div,
        }
    }
}

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
    #[must_use]
    pub fn new(num_levels: usize, bots: &'a mut [T], sorter: S) -> TreeBuildVisitor<'a, T, S> {
        assert!(num_levels >= 1);
        TreeBuildVisitor {
            bots,
            current_height: num_levels - 1,
            sorter,
            is_xaxis: true,
        }
    }
    #[must_use]
    pub fn get_height(&self) -> usize {
        self.current_height
    }
    #[must_use]
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

#[derive(Copy, Clone, Default)]
pub struct DefaultSorter;

impl Sorter for DefaultSorter {
    fn sort(&self, axis: impl Axis, bots: &mut [impl Aabb]) {
        crate::util::sweeper_update(axis, bots);
    }
}

#[derive(Copy, Clone, Default)]
pub struct NoSorter;

impl Sorter for NoSorter {
    fn sort(&self, _axis: impl Axis, _bots: &mut [impl Aabb]) {}
}

pub use axgeom::Range;
pub use axgeom::Rect;

/// The underlying number type used for the tree.
/// It is auto implemented by all types that satisfy the type constraints.
/// Notice that no arithmetic is possible. The tree is constructed
/// using only comparisons and copying.
pub trait Num: PartialOrd + Copy + Default + std::fmt::Debug {}
impl<T> Num for T where T: PartialOrd + Copy + Default + std::fmt::Debug {}

///
/// Trait to signify that this object has an axis aligned bounding box.
///
pub trait Aabb {
    type Num: Num;
    fn get(&self) -> &Rect<Self::Num>;
}

impl<T: Aabb> Aabb for &T {
    type Num = T::Num;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        T::get(self)
    }
}
impl<T: Aabb> Aabb for &mut T {
    type Num = T::Num;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        T::get(self)
    }
}

impl<N: Num> Aabb for Rect<N> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        self
    }
}

/// A bounding box container object that implements [`Aabb`] and [`HasInner`].
/// Note that `&mut BBox<N,T>` also implements [`Aabb`] and [`HasInner`].
///
/// Using this one struct the user can construct the following types for bboxes to be inserted into the tree:
///
///* `BBox<N,T>`  (direct)
///* `&mut BBox<N,T>` (indirect)
///* `BBox<N,&mut T>` (rect direct, T indirect)
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct BBox<N, T> {
    pub rect: Rect<N>,
    pub inner: T,
}

impl<N, T> BBox<N, T> {
    /// Constructor. Also consider using [`crate::bbox()`]
    #[inline(always)]
    #[must_use]
    pub fn new(rect: Rect<N>, inner: T) -> BBox<N, T> {
        BBox { rect, inner }
    }
}

impl<N: Num, T> Aabb for BBox<N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        &self.rect
    }
}

impl<N: Num, T> HasInner for BBox<N, T> {
    type Inner = T;
    #[inline(always)]
    fn destruct_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner) {
        (&self.rect, &mut self.inner)
    }
}

impl<N: Num, T> HasInner for &mut BBox<N, T> {
    type Inner = T;
    #[inline(always)]
    fn destruct_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner) {
        (&self.rect, &mut self.inner)
    }
}

///
/// BBox with a reference.
///
/// Similar to `BBox<N,&mut T>` except
/// get_inner_mut() doesnt return a `&mut &mut T`
/// but instead just a `&mut T`.
///
pub struct BBoxMut<'a, N, T> {
    pub rect: axgeom::Rect<N>,
    pub inner: &'a mut T,
}

impl<'a, N, T> BBoxMut<'a, N, T> {
    /// Constructor. Also consider using [`crate::bbox()`]
    #[inline(always)]
    #[must_use]
    pub fn new(rect: Rect<N>, inner: &'a mut T) -> BBoxMut<'a, N, T> {
        BBoxMut { rect, inner }
    }
}
impl<N: Num, T> Aabb for BBoxMut<'_, N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &axgeom::Rect<N> {
        &self.rect
    }
}
impl<N: Num, T> HasInner for BBoxMut<'_, N, T> {
    type Inner = T;
    #[inline(always)]
    fn destruct_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner) {
        (&self.rect, self.inner)
    }
}

/// When we traverse the tree in read-only mode, we can simply return a reference to each node.
/// We don't need to protect the user from only mutating parts of the BBox's since they can't
/// change anything.
pub type Vistr<'a, N> = compt::dfs_order::Vistr<'a, N, compt::dfs_order::PreOrder>;

mod vistr_mut {
    use crate::*;

    /// Tree Iterator that returns a protected mutable reference to each node.
    #[repr(transparent)]
    pub struct VistrMutPin<'a, N> {
        inner: compt::dfs_order::VistrMut<'a, N, compt::dfs_order::PreOrder>,
    }

    impl<'a, N> VistrMutPin<'a, N> {
        #[inline(always)]
        pub(crate) fn new(
            inner: compt::dfs_order::VistrMut<'a, N, compt::dfs_order::PreOrder>,
        ) -> Self {
            VistrMutPin { inner }
        }
        /// It is safe to borrow the iterator and then produce mutable references from that
        /// as long as by the time the borrow ends, all the produced references also go away.
        #[inline(always)]
        pub fn borrow_mut(&mut self) -> VistrMutPin<N> {
            VistrMutPin {
                inner: self.inner.borrow_mut(),
            }
        }

        #[inline(always)]
        pub fn borrow(&self) -> Vistr<N> {
            self.inner.borrow()
        }

        #[inline(always)]
        pub fn get_height(&self) -> usize {
            compt::FixedDepthVisitor::get_height(self)
        }

        #[inline(always)]
        pub fn into_slice(self) -> AabbPin<&'a mut [N]> {
            AabbPin::new(self.inner.into_slice())
        }
    }

    impl<'a, N> compt::FixedDepthVisitor for VistrMutPin<'a, N> {}

    impl<'a, N> Visitor for VistrMutPin<'a, N> {
        type Item = AabbPin<&'a mut N>;

        #[inline(always)]
        fn next(self) -> (Self::Item, Option<[Self; 2]>) {
            let (nn, rest) = self.inner.next();

            let k = rest
                .map(|[left, right]| [VistrMutPin { inner: left }, VistrMutPin { inner: right }]);

            (AabbPin::new(nn), k)
        }

        #[inline(always)]
        fn level_remaining_hint(&self) -> (usize, Option<usize>) {
            self.inner.level_remaining_hint()
        }

        #[inline(always)]
        fn dfs_preorder(self, mut func: impl FnMut(Self::Item)) {
            self.inner.dfs_preorder(move |a| func(AabbPin::new(a)));
        }
    }
}
pub use vistr_mut::VistrMutPin;

/// A node in [`Tree`].
#[repr(C)]
pub struct Node<'a, T: Aabb> {
    pub range: AabbPin<&'a mut [T]>,

    // if range is empty, then value is unspecified.
    // if range is not empty, then cont can be read.
    pub cont: axgeom::Range<T::Num>,

    // for non leafs:
    //   if there is a bot either in this node or in a child node, then div is some.
    //
    // for leafs:
    //   value is none
    pub div: Option<T::Num>,
}

impl<'a, T: Aabb> HasElem for Node<'a, T> {
    type T = T;
    fn get_elems(&mut self) -> AabbPin<&mut [T]> {
        self.range.borrow_mut()
    }
}
pub trait HasElem {
    type T;
    fn get_elems(&mut self) -> AabbPin<&mut [Self::T]>;
}

#[derive(Debug, Clone)]
pub struct NodeData<N: Num> {
    pub range: usize,
    pub cont: axgeom::Range<N>,
    pub div: Option<N>,
}
