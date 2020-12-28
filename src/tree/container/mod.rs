//! Container trees that deref to [`Tree`]
//!
//! Most of the time using [`Tree`] is enough. But in certain cases
//! we want more control. This module provides [`TreeRef`] and [`TreeRefInd`]
//!
use super::*;

mod collect;
pub use self::collect::*;
use alloc::boxed::Box;
mod inner;

#[repr(transparent)]
struct Ptr<T: ?Sized>(*mut T);
impl<T: ?Sized> Copy for Ptr<T> {}

impl<T: ?Sized> Clone for Ptr<T> {
    #[inline(always)]
    fn clone(&self) -> Ptr<T> {
        *self
    }
}
unsafe impl<T: ?Sized> Send for Ptr<T> {}
unsafe impl<T: ?Sized> Sync for Ptr<T> {}

/// A less general tree that providess `collect` functions
/// and also derefs to a [`Tree`].
///
/// [`TreeRefInd`] assumes there is a layer of indirection where
/// all the pointers point to the same slice.
/// It uses this assumption to provide `collect` functions that allow
/// storing query results that can then be iterated through multiple times
/// quickly.
///
pub struct TreeRefInd<'a, A: Axis, N: Num, T> {
    tree: inner::TreeIndInner<A, N, T>,
    _p: PhantomData<&'a mut (T, N)>,
}

impl<'a, N: Num, T> TreeRefInd<'a, DefaultA, N, T> {
    pub fn new(
        arr: &'a mut [T],
        func: impl FnMut(&mut T) -> Rect<N>,
    ) -> TreeRefInd<'a, DefaultA, N, T> {
        Self::with_axis(default_axis(), arr, func)
    }
}

impl<'a, N: Num + Send + Sync, T: Send + Sync> TreeRefInd<'a, DefaultA, N, T> {
    pub fn new_par(
        arr: &'a mut [T],
        func: impl FnMut(&mut T) -> Rect<N>,
    ) -> TreeRefInd<'a, DefaultA, N, T> {
        Self::with_axis_par(default_axis(), arr, func)
    }
}

impl<'a, A: Axis, N: Num + Send + Sync, T: Send + Sync> TreeRefInd<'a, A, N, T> {
    pub fn with_axis_par(
        axis: A,
        arr: &'a mut [T],
        func: impl FnMut(&mut T) -> Rect<N>,
    ) -> TreeRefInd<'a, A, N, T> {
        TreeRefInd {
            tree: inner::TreeIndInner::with_axis_par(axis, arr, func),
            _p: PhantomData,
        }
    }
}
impl<'a, A: Axis, N: Num, T> TreeRefInd<'a, A, N, T> {
    pub fn with_axis(
        axis: A,
        arr: &'a mut [T],
        func: impl FnMut(&mut T) -> Rect<N>,
    ) -> TreeRefInd<'a, A, N, T> {
        TreeRefInd {
            tree: inner::TreeIndInner::with_axis(axis, arr, func),
            _p: PhantomData,
        }
    }

    //TODO doc
    pub fn as_tree_ref_mut(&mut self)->&mut TreeRef<'a,A,BBox<N, &'a mut T>>{
        &mut *self
    }

    /// ```rust
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut k=[4];
    /// let mut b=broccoli::container::TreeRefInd::new(&mut k,|&mut d|rect(d,d,d,d));
    /// b.find_colliding_pairs_mut(|a,b|{});
    /// assert_eq!(b.get_elements()[0],4);
    /// b.find_colliding_pairs_mut(|a,b|{});
    /// ```
    pub fn get_elements(&self) -> &[T] {
        unsafe { &*self.tree.orig.0 }
    }

    /// ```rust
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut k=[0];
    /// let mut b=broccoli::container::TreeRefInd::new(&mut k,|&mut d|rect(d,d,d,d));
    /// b.find_colliding_pairs_mut(|a,b|{});
    /// b.get_elements_mut()[0]=5;
    /// b.find_colliding_pairs_mut(|a,b|{});
    /// ```
    pub fn get_elements_mut(&mut self) -> &'a mut [T] {
        unsafe { &mut *self.tree.orig.0 }
    }
}

impl<'a, A: Axis, N: Num + 'a, T> core::ops::Deref for TreeRefInd<'a, A, N, T> {
    type Target = TreeRef<'a, A, BBox<N, &'a mut T>>;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.tree.inner.as_tree() as *const _ as *const _) }
    }
}
impl<'a, A: Axis, N: Num + 'a, T> core::ops::DerefMut for TreeRefInd<'a, A, N, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.tree.inner.as_tree_mut() as *mut _ as *mut _) }
    }
}

/// Provides a function to allow the user to ger the original slice of
/// elements (sorted by the tree). Derefs to [`Tree`].
///
/// With the regular [`Tree`], you can't
/// get access to the underlying list of elements after
/// the tree has been constructed without destroying the tree.
///
/// ```rust
/// use broccoli::{prelude::*,bbox,rect};
/// let mut k=[bbox(rect(0,10,0,10),8)];
/// let mut b=broccoli::new(&mut k);
/// b.find_colliding_pairs_mut(|a,b|{});
/// k[0].inner=4;    
/// // b.find_colliding_pairs_mut(|a,b|{}); //<---can't use tree again
/// ```
/// With a regular [`Tree`] you can certainly iterate over all
/// elements in the tree by using `vistr_mut`, but you can't get
/// a slice to the whole continguous list of elements.
///
/// `TreeRef` provides the function [`TreeRef::get_bbox_elements_mut`]
/// that allows the user to get mutable access to the underlying slice of elements.
/// The elements are in the order determined during construction of the tree, i.e.
/// its not the same as the original order passed in. The user is also forbidden from
/// mutating the aabbs of each element. Only whats inside of it besides the aabb can
/// be mutated.
///
///
/// There's not really many usecases where the user would need to use this function, though.
/// Especially since you can already iterate over all elements mutably by calling `vistr_mut`.
///
///```
/// use broccoli::{prelude::*,bbox,rect};
/// let mut bots = [bbox(rect(0,10,0,10),0)];
/// let mut tree = broccoli::new(&mut bots);
///
/// use compt::Visitor;
/// for b in tree.vistr_mut().dfs_preorder_iter().flat_map(|n|n.into_range().iter_mut()){
///    *b.unpack_inner()+=1;    
/// }
/// assert_eq!(bots[0].inner,1);
///```
///
/// However it is useful to implement the [`crate::analyze::NaiveCheck`]
///
#[repr(transparent)]
pub struct TreeRef<'a, A: Axis, T: Aabb> {
    tree: inner::TreeRefInner<A, T>,
    _p: PhantomData<&'a mut T>,
}

use crate::query::NaiveComparable;
impl<'a, A: Axis+'a, T: Aabb> NaiveComparable<'a> for TreeRef<'a, A, T> {
    type K=Tree<'a,A,T>;
    type T = T;
    type Num = T::Num;
    fn get_tree(&mut self) -> &mut Tree<'a,A,T>{
        &mut *self
        
    }
    fn get_elements_mut(&mut self)->PMut<[T]>{
        self.get_bbox_elements_mut()
    }
}

impl<'a, A: Axis, T: Aabb> core::ops::Deref for TreeRef<'a, A, T> {
    type Target = Tree<'a, A, T>;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(&self.tree.inner as *const _ as *const _) }
    }
}
impl<'a, A: Axis, T: Aabb> core::ops::DerefMut for TreeRef<'a, A, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(&mut self.tree.inner as *mut _ as *mut _) }
    }
}

impl<'a, T: Aabb> TreeRef<'a, DefaultA, T> {
    pub fn new(arr: &'a mut [T]) -> TreeRef<'a, DefaultA, T> {
        TreeRef::with_axis(default_axis(), arr)
    }
}

impl<'a, T: Aabb + Send + Sync> TreeRef<'a, DefaultA, T>
where
    T::Num: Send + Sync,
{
    pub fn new_par(arr: &'a mut [T]) -> TreeRef<'a, DefaultA, T> {
        TreeRef::with_axis_par(default_axis(), arr)
    }
}

impl<'a, A: Axis, T: Aabb + Send + Sync> TreeRef<'a, A, T>
where
    T::Num: Send + Sync,
{
    pub fn with_axis_par(a: A, arr: &'a mut [T]) -> TreeRef<'a, A, T> {
        TreeRef {
            tree: inner::TreeRefInner::with_axis_par(a, arr),
            _p: PhantomData,
        }
    }
}

impl<'a, A: Axis, T: Aabb> TreeRef<'a, A, T> {
    pub fn with_axis(a: A, arr: &'a mut [T]) -> TreeRef<'a, A, T> {
        TreeRef {
            tree: inner::TreeRefInner::with_axis(a, arr),
            _p: PhantomData,
        }
    }

    /// ```rust
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut k=[bbox(rect(0,10,0,10),8)];
    /// let mut b=broccoli::container::TreeRef::new(&mut k);
    /// b.find_colliding_pairs_mut(|a,b|{});
    /// assert_eq!(b.get_bbox_elements()[0].inner,8);
    /// b.find_colliding_pairs_mut(|a,b|{});
    /// ```
    ///
    pub fn get_bbox_elements(&self) -> &[T] {
        unsafe { &*self.tree.orig.0 }
    }

    /// ```rust
    /// use broccoli::{prelude::*,bbox,rect};
    /// let mut k=[bbox(rect(0,10,0,10),8)];
    /// let mut b=broccoli::container::TreeRef::new(&mut k);
    /// b.find_colliding_pairs_mut(|a,b|{});
    /// *b.get_bbox_elements_mut().get_index_mut(0).unpack_inner()=5;
    /// b.find_colliding_pairs_mut(|a,b|{});
    /// ```
    ///
    pub fn get_bbox_elements_mut(&mut self) -> PMut<'a, [T]> {
        PMut::new(unsafe { &mut *self.tree.orig.0 })
    }
}

/// An owned version of [`TreeRefInd`]
///
/// ```rust
/// use axgeom::*;
/// use broccoli::{*,container::*,analyze::DefaultA};
///
/// fn not_lifetimed()->TreeOwnedInd<DefaultA,i32,Vec2<i32>>
/// {
///     let rect=vec![vec2(0,10),vec2(3,30)].into_boxed_slice();
///     TreeOwnedInd::new(rect,|&mut p|{
///         let radius=vec2(10,10);
///         Rect::from_point(p,radius)
///     })
/// }
///
/// not_lifetimed();
///
/// ```
pub struct TreeOwnedInd<A: Axis, N: Num, T> {
    tree: inner::TreeIndInner<A, N, T>,
    _bots: Box<[T]>,
}

impl<N: Num + Send + Sync, T: Send + Sync> TreeOwnedInd<DefaultA, N, T> {
    pub fn new_par(
        bots: Box<[T]>,
        func: impl FnMut(&mut T) -> Rect<N>,
    ) -> TreeOwnedInd<DefaultA, N, T> {
        TreeOwnedInd::with_axis_par(default_axis(), bots, func)
    }
}
impl<A: Axis, N: Num + Send + Sync, T: Send + Sync> TreeOwnedInd<A, N, T> {
    pub fn with_axis_par(
        axis: A,
        mut bots: Box<[T]>,
        func: impl FnMut(&mut T) -> Rect<N>,
    ) -> TreeOwnedInd<A, N, T> {
        TreeOwnedInd {
            tree: inner::TreeIndInner::with_axis_par(axis, &mut bots, func),
            _bots: bots,
        }
    }
}

impl<N: Num, T> TreeOwnedInd<DefaultA, N, T> {
    pub fn new(
        bots: Box<[T]>,
        func: impl FnMut(&mut T) -> Rect<N>,
    ) -> TreeOwnedInd<DefaultA, N, T> {
        Self::with_axis(default_axis(), bots, func)
    }
}
impl<A: Axis, N: Num, T> TreeOwnedInd<A, N, T> {
    ///Create an owned Tree in one thread.
    pub fn with_axis(
        axis: A,
        mut bots: Box<[T]>,
        func: impl FnMut(&mut T) -> Rect<N>,
    ) -> TreeOwnedInd<A, N, T> {
        TreeOwnedInd {
            tree: inner::TreeIndInner::with_axis(axis, &mut bots, func),
            _bots: bots,
        }
    }
    ///Cant use Deref because of lifetime
    pub fn as_tree(&self) -> &TreeRefInd<A, N, T> {
        unsafe { &*(&self.tree as *const _ as *const _) }
    }

    ///Cant use Deref because of lifetime
    pub fn as_tree_mut(&mut self) -> &mut TreeRefInd<A, N, T> {
        unsafe { &mut *(&mut self.tree as *mut _ as *mut _) }
    }
}

/// An owned version of [`TreeRef`]
///
/// An owned `(Rect<N>,T)` example
///
/// ```rust
/// use broccoli::{node::BBox,bbox,rect,prelude::*,container::*,analyze::DefaultA};
///
/// fn not_lifetimed()->TreeOwned<DefaultA,BBox<i32,f32>>
/// {
///     let a=vec![bbox(rect(0,10,0,10),0.0)].into_boxed_slice();
///     TreeOwned::new(a)
/// }
///
/// not_lifetimed();
///
/// ```
pub struct TreeOwned<A: Axis, T: Aabb> {
    tree: inner::TreeRefInner<A, T>,
    _bots: Box<[T]>,
}

impl<T: Aabb + Send + Sync> TreeOwned<DefaultA, T>
where
    T::Num: Send + Sync,
{
    pub fn new_par(bots: Box<[T]>) -> TreeOwned<DefaultA, T> {
        TreeOwned::with_axis_par(default_axis(), bots)
    }
}
impl<A: Axis, T: Aabb + Send + Sync> TreeOwned<A, T>
where
    T::Num: Send + Sync,
{
    pub fn with_axis_par(axis: A, mut bots: Box<[T]>) -> TreeOwned<A, T> {
        TreeOwned {
            tree: inner::TreeRefInner::with_axis_par(axis, &mut bots),
            _bots: bots,
        }
    }
}
impl<T: Aabb> TreeOwned<DefaultA, T> {
    pub fn new(bots: Box<[T]>) -> TreeOwned<DefaultA, T> {
        Self::with_axis(default_axis(), bots)
    }
}
impl<A: Axis, T: Aabb> TreeOwned<A, T> {
    ///Create an owned Tree in one thread.
    pub fn with_axis(axis: A, mut bots: Box<[T]>) -> TreeOwned<A, T> {
        TreeOwned {
            tree: inner::TreeRefInner::with_axis(axis, &mut bots),
            _bots: bots,
        }
    }

    ///Cant use Deref because of lifetime
    pub fn as_tree(&self) -> &TreeRef<A, T> {
        unsafe { &*(&self.tree as *const _ as *const _) }
    }

    ///Cant use Deref because of lifetime
    pub fn as_tree_mut(&mut self) -> &mut TreeRef<A, T> {
        unsafe { &mut *(&mut self.tree as *mut _ as *mut _) }
    }
}
