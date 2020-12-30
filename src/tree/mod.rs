use crate::inner_prelude::*;

#[cfg(test)]
mod tests;

pub mod build;
use build::TreeBuilder;

pub mod container;

struct TreeInner<N> {
    inner: compt::dfs_order::CompleteTreeContainer<N, compt::dfs_order::PreOrder>,
}

///The data structure this crate revoles around.
#[repr(transparent)]
pub struct Tree<'a, T: Aabb> {
    inner: TreeInner<Node<'a, T>>,
}

///Create a [`Tree`].
///
/// # Examples
///
///```
/// let mut bots = [axgeom::rect(0,10,0,10)];
/// let tree = broccoli::new(&mut bots);
///
///```
pub fn new<T: Aabb>(bots: &mut [T]) -> Tree<T> {
    Tree::new(bots)
}

///Create a [`Tree`] in parallel.
///
/// # Examples
///
///```
/// let mut bots = [axgeom::rect(0,10,0,10)];
/// let tree = broccoli::new_par(&mut bots);
///
///```
pub fn new_par<T: Aabb + Send + Sync>(bots: &mut [T]) -> Tree<T>
where
    T::Num: Send + Sync,
{
    Tree::new_par(bots)
}

impl<'a, T: Aabb> NbodyQuery<'a> for Tree<'a, T> {}
impl<'a, T: Aabb> DrawQuery<'a> for Tree<'a, T> {}
impl<'a, T: Aabb> IntersectQuery<'a> for Tree<'a, T> {}
impl<'a, T: Aabb> RectQuery<'a> for Tree<'a, T> {}
impl<'a, T: Aabb> ColfindQuery<'a> for Tree<'a, T> {}
impl<'a, T: Aabb> RaycastQuery<'a> for Tree<'a, T> {}
impl<'a, T: Aabb> KnearestQuery<'a> for Tree<'a, T> {}

impl<'a, T: Aabb> Queries<'a> for Tree<'a, T> {
    type T = T;
    type Num = T::Num;

    #[inline(always)]
    fn vistr_mut(&mut self) -> VistrMut<Node<'a, T>> {
        VistrMut::new(self.inner.inner.vistr_mut())
    }

    #[inline(always)]
    fn vistr(&self) -> Vistr<Node<'a, T>> {
        self.inner.inner.vistr()
    }
}

impl<'a, T: Aabb> Tree<'a, T> {
    ///Create a [`Tree`].
    ///
    /// # Examples
    ///
    ///```
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let tree = broccoli::Tree::new(&mut bots);
    ///
    ///```
    pub fn new(bots: &'a mut [T]) -> Tree<'a, T> {
        TreeBuilder::new(bots).build_seq()
    }
    ///Create a [`Tree`] in parallel.
    ///
    /// # Examples
    ///
    ///```
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let tree = broccoli::Tree::new_par(&mut bots);
    ///
    ///```
    pub fn new_par(bots: &'a mut [T]) -> Tree<'a, T>
    where
        T: Send + Sync,
        T::Num: Send + Sync,
    {
        TreeBuilder::new(bots).build_par()
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::build;
    /// const NUM_ELEMENT:usize=40;
    /// let mut bots = [axgeom::rect(0,10,0,10);NUM_ELEMENT];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.get_height(),build::TreePreBuilder::new(NUM_ELEMENT).get_height());
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
    /// use broccoli::build;
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.num_nodes(),build::TreePreBuilder::new(1).num_nodes());
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
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.get_nodes()[0].range[0], axgeom::rect(0,10,0,10));
    ///
    ///```
    #[must_use]
    pub fn get_nodes(&self) -> &[Node<'a, T>] {
        self.inner.inner.get_nodes()
    }

    /// # Examples
    ///
    ///```
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.get_nodes_mut().get_index_mut(0).range[0], axgeom::rect(0,10,0,10));
    ///
    ///```
    #[must_use]
    pub fn get_nodes_mut(&mut self) -> PMut<[Node<'a, T>]> {
        PMut::new(self.inner.inner.get_nodes_mut())
    }
}
