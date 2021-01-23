use super::*;

/// An owned version of [`TreeInd`]
///
/// ```rust
/// use axgeom::*;
/// use broccoli::{*,container::*,node::*,prelude::*};
///
/// fn not_lifetimed()->TreeIndOwned<i32,BBox<i32,f32>>
/// {
///     let rect=vec![bbox(rect(0,10,0,10),0.0)].into_boxed_slice();
///     TreeIndOwned::new(rect,|b|{
///         b.rect
///     })
/// }
///
/// let mut tree=not_lifetimed();
/// 
/// let mut pairs = tree.as_tree_mut().collect_colliding_pairs(|a,b|Some(()));
///
/// ```
pub struct TreeIndOwned<N: Num, T> {
    tree: TreeIndPtr<N,T>,
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

impl<N: Num + Send + Sync, T: Send + Sync> TreeIndOwned<N, T> {
    pub fn new_par(mut bots: Box<[T]>,joiner:impl crate::Joinable, func: impl FnMut(&mut T) -> Rect<N>) -> TreeIndOwned<N, T> {
        
        let mut base=TreeIndBase::new(&mut bots,func);
        let tree=base.build_par(joiner);
        let tree=tree.into_ptr();
        let _base=convert_box(base.into_inner());
        
        TreeIndOwned {
            tree,
            _bots: bots,
            _base
        }
    }
}
impl<N: Num, T> TreeIndOwned<N, T> {
    pub fn new(mut bots: Box<[T]>, func: impl FnMut(&mut T) -> Rect<N>) -> TreeIndOwned<N, T> {
        
        let mut base=TreeIndBase::new(&mut bots,func);
        let tree=base.build();
        let tree=tree.into_ptr();
        let _base=convert_box(base.into_inner());
        
        TreeIndOwned {
            tree,
            _bots: bots,
            _base
        }
    }
}

impl<N: Num, T> TreeIndOwned<N, T> {
    ///Cant use Deref because of lifetime
    #[inline(always)]
    pub fn as_tree<'a,'b,'c>(&'c self) -> &'c TreeInd<'a,'b,N, T> {
        unsafe { &*(&self.tree as *const TreeIndPtr<_,_> as *const TreeInd<_,_>) }
    }

    ///Cant use Deref because of lifetime
    #[inline(always)]
    pub fn as_tree_mut<'a,'b,'c>(&'c mut self) -> &'c mut TreeInd<'a,'b,N, T> {
        unsafe { &mut *(&mut self.tree as *mut TreeIndPtr<_,_> as *mut TreeInd<_,_>) }
    }
}


/// An owned version of [`Tree`](crate::Tree)
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
/// let mut tree = not_lifetimed();
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
    pub fn new_par(joiner:impl crate::Joinable,mut bots: Box<[T]>) -> TreeOwned<T> {
        let tree = crate::new_par(joiner,&mut bots);

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
