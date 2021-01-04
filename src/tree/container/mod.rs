//! Container trees that deref to [`Tree`]
//!
//! Most of the time using [`Tree`] is enough. But in certain cases
//! we want more control. 

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
pub struct TreeRefInd<'a, N: Num, T> {
    tree: inner::TreeIndInner<N, T>,
    _p: PhantomData<Tree<'a, BBox<N, &'a mut T>>>,
}

impl<'a, N: Num, T> TreeRefInd<'a, N, T> {
    pub fn new(arr: &'a mut [T], func: impl FnMut(&mut T) -> Rect<N>) -> TreeRefInd<'a, N, T> {
        TreeRefInd {
            tree: inner::TreeIndInner::new(arr, func),
            _p: PhantomData,
        }
    }
}

impl<'a, N: Num + Send + Sync, T: Send + Sync> TreeRefInd<'a, N, T> {
    pub fn new_par(arr: &'a mut [T], func: impl FnMut(&mut T) -> Rect<N>) -> TreeRefInd<'a, N, T> {
        TreeRefInd {
            tree: inner::TreeIndInner::new_par(arr, func),
            _p: PhantomData,
        }
    }
}

impl<'a, N: Num, T> TreeRefInd<'a, N, T> {
    ///Explicitly DerefMut.
    pub fn as_tree_ref_mut(&mut self) -> &mut Tree<'a, BBox<N, &'a mut T>> {
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

impl<'a, N: Num + 'a, T> core::ops::Deref for TreeRefInd<'a, N, T> {
    type Target = Tree<'a, BBox<N, &'a mut T>>;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.tree.inner.as_tree() as *const _ as *const _) }
    }
}
impl<'a, N: Num + 'a, T> core::ops::DerefMut for TreeRefInd<'a, N, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.tree.inner.as_tree_mut() as *mut _ as *mut _) }
    }
}


/*
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
/// use broccoli::{prelude::*,bbox,rect,query::Queries};
/// let mut bots = [bbox(rect(0,10,0,10),0)];
/// let mut tree = broccoli::new(&mut bots);
///
/// use compt::Visitor;
/// for b in tree.vistr_mut().dfs_preorder_iter().flat_map(|n|n.into_range().iter_mut()){
///    *b.unpack_inner()+=1;    
/// }
/// assert_eq!(bots[0].inner,1);
///```
/// It is useful to implement functions like [`knearest::assert_k_nearest_mut`](crate::query::knearest::assert_k_nearest_mut)
/// That let you cross check a tree against the naive implementation without destroying the tree.
///
#[repr(C)]
pub struct TreeRef<'a, T: Aabb> {
    tree: crate::Tree<'a, T>,
    orig: Ptr<[T]>,
}

impl<'a, T: Aabb> core::ops::Deref for TreeRef<'a, T> {
    type Target = Tree<'a, T>;
    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}
impl<'a, T: Aabb> core::ops::DerefMut for TreeRef<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tree
    }
}

impl<'a, T: Aabb> TreeRef<'a, T> {
    pub fn new(arr: &'a mut [T]) -> TreeRef<'a, T> {
        let orig = Ptr(arr as *mut _);

        TreeRef {
            tree: crate::new(arr),
            orig,
        }
    }
}

impl<'a, T: Aabb + Send + Sync> TreeRef<'a, T>
where
    T::Num: Send + Sync,
{
    pub fn new_par(arr: &'a mut [T]) -> TreeRef<'a, T> {
        let orig = Ptr(arr as *mut _);

        TreeRef {
            tree: crate::new_par(arr),
            orig,
        }
    }
}

/*
impl<'a, T: Aabb> From<TreeRef<'a, T>> for Tree<'a, T> {
    fn from(a: TreeRef<'a, T>) -> Self {
        Tree {
            inner: a.tree.inner,
            num_aabbs:a.orig.len()
        }
    }
}
*/

impl<'a, T: Aabb> TreeRef<'a, T> {
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
        unsafe { &*self.orig.0 }
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
        PMut::new(unsafe { &mut *self.orig.0 })
    }
}
*/

/// An owned version of [`TreeRefInd`]
///
/// ```rust
/// use axgeom::*;
/// use broccoli::{*,container::*};
///
/// fn not_lifetimed()->TreeOwnedInd<i32,Vec2<i32>>
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
pub struct TreeOwnedInd<N: Num, T> {
    tree: inner::TreeIndInner<N, T>,
    _bots: Box<[T]>,
}

impl<N: Num + Send + Sync, T: Send + Sync> TreeOwnedInd<N, T> {
    pub fn new_par(mut bots: Box<[T]>, func: impl FnMut(&mut T) -> Rect<N>) -> TreeOwnedInd<N, T> {
        TreeOwnedInd {
            tree: inner::TreeIndInner::new_par(&mut bots, func),
            _bots: bots,
        }
    }
}
impl<N: Num, T> TreeOwnedInd<N, T> {
    pub fn new(mut bots: Box<[T]>, func: impl FnMut(&mut T) -> Rect<N>) -> TreeOwnedInd<N, T> {
        TreeOwnedInd {
            tree: inner::TreeIndInner::new(&mut bots, func),
            _bots: bots,
        }
    }
}

impl<N: Num, T> TreeOwnedInd<N, T> {
    ///Cant use Deref because of lifetime
    pub fn as_tree(&self) -> &TreeRefInd<N, T> {
        unsafe { &*(&self.tree as *const _ as *const _) }
    }

    ///Cant use Deref because of lifetime
    pub fn as_tree_mut(&mut self) -> &mut TreeRefInd<N, T> {
        unsafe { &mut *(&mut self.tree as *mut _ as *mut _) }
    }
}


/// An owned version of [`crate::Tree`]
///
/// An owned `(Rect<N>,T)` example
///
/// ```rust
/// use broccoli::{node::BBox,bbox,rect,prelude::*,container::*};
///
/// fn not_lifetimed()->TreeOwned<BBox<i32,f32>>
/// {
///     let a=vec![bbox(rect(0,10,0,10),0.0)].into_boxed_slice();
///     TreeOwned::new(a)
/// }
///
/// not_lifetimed();
///
/// ```
#[repr(C)]
pub struct TreeOwned<T: Aabb> {
    inner: TreePtr<T>,
    _bots: Box<[T]>,
}

impl<T: Aabb + Send + Sync> TreeOwned<T>
where
    T::Num: Send + Sync,
{
    pub fn new_par(mut bots: Box<[T]>) -> TreeOwned<T> {
        let inner = inner::make_owned_par(&mut bots);

        TreeOwned {
            inner,
            _bots: bots,
        }
    }
}

impl<T: Aabb> TreeOwned<T> {
    pub fn new(mut bots: Box<[T]>) -> TreeOwned<T> {
        let inner = inner::make_owned(&mut bots);
        TreeOwned {
            inner,
            _bots: bots,
        }
    }
}
impl<T: Aabb> TreeOwned<T> {
    ///Cant use Deref because of lifetime
    pub fn as_tree(&self) -> &Tree<T> {
        unsafe { &*(&self.inner as *const _ as *const _) }
    }

    ///Cant use Deref because of lifetime
    pub fn as_tree_mut(&mut self) -> &mut Tree<T> {
        unsafe { &mut *(&mut self.inner as *mut _ as *mut _) }
    }
}
