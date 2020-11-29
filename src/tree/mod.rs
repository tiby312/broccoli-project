use crate::inner_prelude::*;

#[cfg(test)]
mod tests;

pub mod analyze;

///Contains code to write generic code that can be run in parallel, or sequentially. The api is exposed
///in case users find it useful when writing parallel query code to operate on the tree.
pub(crate) mod par;

//mod notsorted;
//pub use self::notsorted::NotSorted;

use analyze::TreeBuilder;

pub(crate) use self::node::*;

///Contains node-level building block structs and visitors used for a [`Tree`].
pub mod node;

pub mod container;

use crate::query::*;

pub(crate) struct TreeInner<A: Axis, N> {
    axis: A,
    inner: compt::dfs_order::CompleteTreeContainer<N, compt::dfs_order::PreOrder>,
}

///The data structure this crate revoles around.
#[repr(transparent)]
pub struct Tree<'a, A: Axis, T: Aabb> {
    inner: TreeInner<A, NodeMut<'a, T>>,
}

///The default starting axis of a [`Tree`]. It is set to be the `Y` axis.
///This means that the first divider is a horizontal line since it is
///partitioning space based off of the aabb's `Y` value.
pub type DefaultA = YAXIS;

///Returns the default axis type.
pub const fn default_axis() -> YAXIS {
    YAXIS
}

///Create a [`Tree`] using the default axis.
///
/// # Examples
///
///```
/// let mut bots = [axgeom::rect(0,10,0,10)];
/// let tree = broccoli::new(&mut bots);
///
///```
pub fn new<'a, T: Aabb>(bots: &'a mut [T]) -> Tree<'a, DefaultA, T> {
    TreeBuilder::new(bots).build_seq()
}

///Create a [`Tree`] using a specified axis.
///
/// # Examples
///
///```
/// let mut bots = [axgeom::rect(0,10,0,10)];
/// let tree = broccoli::with_axis(axgeom::XAXIS,&mut bots);
///
///```
pub fn with_axis<'a, A: Axis, T: Aabb>(axis: A, bots: &'a mut [T]) -> Tree<'a, A, T> {
    TreeBuilder::with_axis(axis, bots).build_seq()
}

///Create a [`Tree`] using the default axis in parallel.
///
/// # Examples
///
///```
/// let mut bots = [axgeom::rect(0,10,0,10)];
/// let tree = broccoli::new_par(&mut bots);
///
///```
pub fn new_par<'a, T: Aabb + Send + Sync>(bots: &'a mut [T]) -> Tree<'a, DefaultA, T> {
    TreeBuilder::new(bots).build_par()
}

///Create a [`Tree`] using a specified axis in parallel.
///
/// # Examples
///
///```
/// let mut bots = [axgeom::rect(0,10,0,10)];
/// let tree = broccoli::with_axis_par(axgeom::XAXIS,&mut bots);
///
///```
pub fn with_axis_par<'a, A: Axis, T: Aabb + Send + Sync>(
    axis: A,
    bots: &'a mut [T],
) -> Tree<'a, A, T> {
    TreeBuilder::with_axis(axis, bots).build_par()
}

impl<'a, A: Axis, T: Aabb> Queries<'a> for Tree<'a, A, T> {
    type A = A;
    type T = T;
    type Num = T::Num;

    #[inline(always)]
    fn axis(&self) -> Self::A {
        self.inner.axis
    }

    #[inline(always)]
    fn vistr_mut(&mut self) -> VistrMut<NodeMut<'a, T>> {
        VistrMut {
            inner: self.inner.inner.vistr_mut(),
        }
    }

    #[inline(always)]
    fn vistr(&self) -> Vistr<NodeMut<'a, T>> {
        self.inner.inner.vistr()
    }
}

impl<'a, A: Axis, T: Aabb> Tree<'a, A, T> {
    /// # Examples
    ///
    ///```
    /// use broccoli::analyze;
    /// const NUM_ELEMENT:usize=400;
    /// let mut bots = [axgeom::rect(0,10,0,10);NUM_ELEMENT];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.get_height(),analyze::TreePreBuilder::new(NUM_ELEMENT).get_height());
    ///```
    ///
    #[must_use]
    #[inline(always)]
    pub fn get_height(&self) -> usize {
        self.inner.inner.get_height()
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::analyze;
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.num_nodes(),analyze::TreePreBuilder::new(1).num_nodes());
    ///
    ///```
    #[must_use]
    #[warn(deprecated)]
    #[inline(always)]
    pub fn num_nodes(&self) -> usize {
        self.inner.inner.get_nodes().len()
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::analyze;
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.get_nodes()[0].get().bots[0], axgeom::rect(0,10,0,10));
    ///
    ///```
    #[must_use]
    pub fn get_nodes(&self) -> &[NodeMut<'a, T>] {
        self.inner.inner.get_nodes()
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::analyze;
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.get_nodes_mut().get_index_mut(0).get().bots[0], axgeom::rect(0,10,0,10));
    ///
    ///```
    #[must_use]
    pub fn get_nodes_mut(&mut self) -> PMut<[NodeMut<'a, T>]> {
        PMut::new(self.inner.inner.get_nodes_mut())
    }
}
