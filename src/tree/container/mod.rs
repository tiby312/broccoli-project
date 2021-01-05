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
/// let mut tree=not_lifetimed();
/// 
/// let mut pairs = tree.as_tree_mut().collect_colliding_pairs(|a,b|Some(()));
///
/// ```
pub struct TreeOwnedInd<N: Num, T> {
    tree: TreeRefIndPtr<N,T>,
    _base: Box<[BBox<N,Ptr<T>>]>,
    _bots: Box<[T]>,
}
fn convert_box<T,X>(mut v_orig:Box<[T]>)->Box<[X]>{
    assert_eq!(core::mem::size_of::<X>(),core::mem::size_of::<T>());
    assert_eq!(core::mem::align_of::<X>(),core::mem::align_of::<T>());
    unsafe{
        // Ensure the original vector is not dropped.
        let ptr=v_orig.as_mut_ptr();
        let length=v_orig.len();
        core::mem::forget(v_orig);
        Box::from_raw(core::slice::from_raw_parts_mut(ptr as *mut _, length))
    }
}

impl<N: Num + Send + Sync, T: Send + Sync> TreeOwnedInd<N, T> {
    pub fn new_par(mut bots: Box<[T]>, func: impl FnMut(&mut T) -> Rect<N>) -> TreeOwnedInd<N, T> {
        
        let mut base=TreeRefBase::new(&mut bots,func);
        let tree=base.build_par();
        let tree=tree.into_ptr();
        let _base=convert_box(base.into_inner());
        
        TreeOwnedInd {
            tree,
            _bots: bots,
            _base
        }
    }
}
impl<N: Num, T> TreeOwnedInd<N, T> {
    pub fn new(mut bots: Box<[T]>, func: impl FnMut(&mut T) -> Rect<N>) -> TreeOwnedInd<N, T> {
        
        let mut base=TreeRefBase::new(&mut bots,func);
        let tree=base.build();
        let tree=tree.into_ptr();
        let _base=convert_box(base.into_inner());
        
        TreeOwnedInd {
            tree,
            _bots: bots,
            _base
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
/// let mut tree =not_lifetimed();
/// 
/// let mut pairs = tree.as_tree_mut().find_colliding_pairs_mut(|a,b|{});
///
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
