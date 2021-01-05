//! Container trees that deref to [`Tree`]
//!
//! Most of the time using [`Tree`] is enough. But in certain cases
//! we want more control. 

use super::*;

mod collect;
pub use self::collect::*;
use alloc::boxed::Box;

#[repr(transparent)]
pub(crate) struct Ptr<T: ?Sized>(*mut T);
impl<T: ?Sized> Copy for Ptr<T> {}

impl<T: ?Sized> Clone for Ptr<T> {
    #[inline(always)]
    fn clone(&self) -> Ptr<T> {
        *self
    }
}
unsafe impl<T: ?Sized> Send for Ptr<T> {}
unsafe impl<T: ?Sized> Sync for Ptr<T> {}


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
    tree: TreeRefIndPtr<N,T>,
    _bots: Box<[T]>,
}

impl<N: Num + Send + Sync, T: Send + Sync> TreeOwnedInd<N, T> {
    pub fn new_par(mut bots: Box<[T]>, func: impl FnMut(&mut T) -> Rect<N>) -> TreeOwnedInd<N, T> {
        
        let mut base=TreeRefBase::new(&mut bots,func);
        let tree=base.build_par();
        TreeOwnedInd {
            tree: tree.into_ptr(),
            _bots: bots,
        }
    }
}
impl<N: Num, T> TreeOwnedInd<N, T> {
    pub fn new(mut bots: Box<[T]>, func: impl FnMut(&mut T) -> Rect<N>) -> TreeOwnedInd<N, T> {
        
        let mut base=TreeRefBase::new(&mut bots,func);
        let tree=base.build();

        TreeOwnedInd {
            tree: tree.into_ptr(),
            _bots: bots,
        }
    }
}

impl<N: Num, T> TreeOwnedInd<N, T> {
    ///Cant use Deref because of lifetime
    #[inline(always)]
    pub fn as_tree<'a,'b,'c>(&'c self) -> &'c TreeRefInd<'a,'b,N, T> {
        unsafe { &*(&self.tree as *const TreeRefIndPtr<_,_> as *const TreeRefInd<_,_>) }
    }

    ///Cant use Deref because of lifetime
    #[inline(always)]
    pub fn as_tree_mut<'a,'b,'c>(&'c mut self) -> &'c mut TreeRefInd<'a,'b,N, T> {
        unsafe { &mut *(&mut self.tree as *mut TreeRefIndPtr<_,_> as *mut TreeRefInd<_,_>) }
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
        let tree = crate::new_par(&mut bots);

        let inner=TreePtr{
            _inner:unsafe{tree.inner.convert()},
            _num_aabbs:tree.num_aabbs
        };
        TreeOwned {
            inner,
            _bots: bots,
        }
    }
}

impl<T: Aabb> TreeOwned<T> {
    pub fn new(mut bots: Box<[T]>) -> TreeOwned<T> {
        let tree = crate::new(&mut bots);

        let inner=TreePtr{
            _inner:unsafe{tree.inner.convert()},
            _num_aabbs:tree.num_aabbs
        };
        TreeOwned {
            inner,
            _bots: bots,
        }
    }
}
impl<T: Aabb> TreeOwned<T> {
    ///Cant use Deref because of lifetime
    #[inline(always)]
    pub fn as_tree(&self) -> &Tree<T> {
        unsafe { &*(&self.inner as *const _ as *const _) }
    }

    ///Cant use Deref because of lifetime
    #[inline(always)]
    pub fn as_tree_mut(&mut self) -> &mut Tree<T> {
        unsafe { &mut *(&mut self.inner as *mut _ as *mut _) }
    }
}
