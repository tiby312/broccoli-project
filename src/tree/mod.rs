use crate::inner_prelude::*;

#[cfg(test)]
mod tests;

pub mod build;
use build::TreeBuilder;

pub mod container;





type TreeInner<N> = compt::dfs_order::CompleteTreeContainer<N, compt::dfs_order::PreOrder>;

#[repr(C)]
struct TreePtr<T:Aabb>{
    _inner:TreeInner<NodePtr<T>>,
    _num_aabbs:usize
}

///The data structure this crate revoles around.
#[repr(C)]
pub struct Tree<'a, T: Aabb> {
    inner: TreeInner<Node<'a, T>>,
    num_aabbs:usize
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
        VistrMut::new(self.inner.vistr_mut())
    }

    #[inline(always)]
    fn vistr(&self) -> Vistr<Node<'a, T>> {
        self.inner.vistr()
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
        self.inner.get_height()
    }


    /// # Examples
    ///
    ///```
    /// use broccoli::build;
    /// const NUM_ELEMENT:usize=7;
    /// let mut bots = [axgeom::rect(0,10,0,10);NUM_ELEMENT];
    /// let mut tree = broccoli::new(&mut bots);
    /// let inner =tree.into_inner();
    /// assert_eq!(inner.into_nodes().len(),1);
    ///```
    #[must_use]
    pub fn into_inner(self)->compt::dfs_order::CompleteTreeContainer<Node<'a,T>, compt::dfs_order::PreOrder>{
        self.inner
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::build;
    /// const NUM_ELEMENT:usize=7;
    /// let mut bots = [axgeom::rect(0,10,0,10);NUM_ELEMENT];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.num_aabbs(),7);
    ///```
    ///
    #[must_use]
    pub fn num_aabbs(&self)->usize{
        self.num_aabbs
    }

    /// # Examples
    ///
    ///```
    /// use broccoli::build;
    /// const NUM_ELEMENT:usize=7;
    /// let mut bots = [axgeom::rect(0,10,0,10);NUM_ELEMENT];
    /// let mut tree = broccoli::new(&mut bots);
    /// let num_aabbs=tree.num_aabbs();
    /// let inner =tree.into_inner();
    /// let tree=unsafe{broccoli::Tree::from_raw_parts(inner,num_aabbs)};
    /// assert_eq!(tree.num_aabbs(),7);
    ///```
    ///
    /// # Safety
    ///
    /// Unsafe, since the user may pass a number of aabbs
    /// that does not reflect the true number of aabbs in
    /// every node.
    ///
    pub unsafe fn from_raw_parts(inner:compt::dfs_order::CompleteTreeContainer<Node<'a,T>, compt::dfs_order::PreOrder>,num_aabbs:usize)->Self{
        Tree{
            inner,
            num_aabbs
        }
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
        self.inner.get_nodes().len()
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
        self.inner.get_nodes()
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
        PMut::new(self.inner.get_nodes_mut())
    }


    /// Return the underlying slice of aabbs in the order sorted during tree construction.
    ///
    /// # Examples
    ///
    ///```
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let mut tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(*tree.get_elements_mut().get_index_mut(0), axgeom::rect(0,10,0,10));
    ///
    ///```
    #[must_use]
    pub fn get_elements_mut(&mut self)->PMut<[T]>{

        fn foo<'a,T:Aabb>(v:VistrMut<'a,Node<T>>)->PMut<'a,[T]>{
            let mut new_slice = None;

            v.dfs_preorder(|a| {
                if let Some(s) = new_slice.take() {
                    new_slice = Some(crate::pmut::combine_slice(s, a.into_range()));
                } else {
                    new_slice = Some(a.into_range());
                }
            });
            new_slice.unwrap()
        }

        let num_aabbs=self.num_aabbs;
        let ret=foo(self.vistr_mut());
        assert_eq!(ret.len(),num_aabbs);
        ret
    }

    /// Return the underlying slice of aabbs in the order sorted during tree construction.
    ///
    /// # Examples
    ///
    ///```
    /// let mut bots = [axgeom::rect(0,10,0,10)];
    /// let tree = broccoli::new(&mut bots);
    ///
    /// assert_eq!(tree.get_elements()[0], axgeom::rect(0,10,0,10));
    ///
    ///```
    #[must_use]
    pub fn get_elements(&self)->&[T]{

        fn foo<'a,T:Aabb>(v:Vistr<'a,Node<T>>)->&'a [T]{
            let mut new_slice = None;

            v.dfs_preorder(|a| {
                if let Some(s) = new_slice.take() {
                    new_slice = Some(crate::util::combine_slice(s, &a.range));
                } else {
                    new_slice = Some(&a.range);
                }
            });
            new_slice.unwrap()
        }

        let num_aabbs=self.num_aabbs;
        let ret=foo(self.vistr());
        assert_eq!(ret.len(),num_aabbs);
        ret
    }

}
