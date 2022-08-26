//!
//! broccoli tree Node and Node visitor structs
//!

use super::*;

/// When we traverse the tree in read-only mode, we can simply return a reference to each node.
/// We don't need to protect the user from only mutating parts of the BBox's since they can't
/// change anything.
pub type Vistr<'a, N> = compt::dfs_order::Vistr<'a, N, compt::dfs_order::PreOrder>;

mod vistr_mut {
    use super::*;
    use compt::Visitor;

    /// Tree Iterator that returns a protected mutable reference to each node.
    #[repr(transparent)]
    #[must_use]
    pub struct VistrMutPin<'a, N> {
        inner: compt::dfs_order::VistrMut<'a, N, compt::dfs_order::PreOrder>,
    }

    impl<'a, N> VistrMutPin<'a, N> {
        #[inline(always)]
        pub fn new(inner: compt::dfs_order::VistrMut<'a, N, compt::dfs_order::PreOrder>) -> Self {
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
    }
}
pub use vistr_mut::VistrMutPin;

///
/// The node of a broccoli tree.
///
pub struct Node<'a, T, N> {
    /// May or may not be sorted.
    pub range: AabbPin<&'a mut [T]>,

    /// if range is empty, then value is `[default,default]`.
    /// if range is not empty, then cont is the min max bounds in on the y axis (if the node belongs to the x axis).
    pub cont: axgeom::Range<N>,

    /// for non leafs:
    ///   if there is a bot either in this node or in a child node, then div is some.
    ///
    /// for leafs:
    ///   value is none
    pub div: Option<N>,

    ///
    /// The minimum number of elements in a child node.
    /// If the left child has 500 bots, and the right child has 20, then
    /// this value will be 20.
    ///
    /// This is used to determine when to start a parallel task.
    /// Starting a parallel task has overhead so we only want to split
    /// one off if we know that both threads have a decent amount of work
    /// to perform in parallel.
    ///
    pub min_elem: usize,
}
impl<'a, T, N: Num> Node<'a, T, N> {
    pub fn borrow_range(&mut self) -> AabbPin<&mut [T]> {
        self.range.borrow_mut()
    }

    pub fn as_data(&self) -> NodeData<N> {
        NodeData {
            range: self.range.len(),
            cont: self.cont,
            div: self.div,
            min_elem: self.min_elem,
            //num_elem: self.num_elem,
        }
    }
}

///
/// Like [`Node`] except only has the number of elem instead of a slice..
///
#[derive(Debug, Clone)]
pub struct NodeData<N: Num> {
    pub range: usize,
    pub cont: axgeom::Range<N>,
    pub div: Option<N>,
    pub min_elem: usize,
    //pub num_elem: usize,
}
