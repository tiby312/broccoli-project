//!
//! ## An owned `(Rect<N>,T)` example
//!
//! ```rust
//! use dinotree_alg::{*,collections::*};
//! use axgeom::*;
//!
//! fn not_lifetimed()->DinoTreeOwned<DefaultA,BBox<i32,f32>>
//! {
//!     let a=vec![bbox(rect(0,10,0,10),0.0)];
//!     DinoTreeOwned::new(a)
//! }
//!
//! not_lifetimed();
//!
//! ```
//!
//! ## An owned `(Rect<N>,*mut T)` example
//!
//! ```rust
//! use dinotree_alg::{*,collections::*};
//! use axgeom::*;
//!
//! fn not_lifetimed()->DinoTreeOwnedInd<DefaultA,i32,Vec2<i32>>
//! {
//!     let rect=vec![vec2(0,10),vec2(3,30)];
//!     DinoTreeOwnedInd::new(rect,|&p|{
//!         let radius=vec2(10,10);
//!         Rect::from_point(p,radius)
//!     })
//! }
//!
//! not_lifetimed();
//!
//! ```

use super::*;


struct ThreadPtr<T>(*mut T);
unsafe impl<T> Send for ThreadPtr<T>{}
unsafe impl<T> Sync for ThreadPtr<T>{}


pub struct DinoTreeRefInd<'a,A:Axis,N:Num,T>{
    inner:DinoTreeOwned<A,BBox<N,*mut T>>,
    orig:*mut [T],
    _p:PhantomData<&'a mut T>
}

impl<'a,N:Num,T> DinoTreeRefInd<'a,DefaultA,N,T>{
    pub fn new(arr:&'a mut [T],func:impl FnMut(&mut T)->Rect<N>)->DinoTreeRefInd<'a,DefaultA,N,T>{
        DinoTreeRefInd::with_axis(default_axis(),arr,func)
    }
}

impl<'a,A:Axis,N:Num,T> DinoTreeRefInd<'a,A,N,T>{
    pub fn with_axis(axis:A,arr:&'a mut [T],mut func:impl FnMut(&mut T)->Rect<N>)->DinoTreeRefInd<'a,A,N,T>{
        let orig=arr as *mut _;
        let bbox = arr
        .iter_mut()
        .map(|b| BBox::new(func(b), b as *mut _))
        .collect();

        let inner=DinoTreeOwned::with_axis(axis,bbox);

        DinoTreeRefInd{
            inner,
            orig,
            _p:PhantomData
        }
    }
    pub fn get_elements(&self)->&[T]{
        unsafe{&*self.orig}
    }
    pub fn get_elements_mut(&mut self)->&'a mut [T]{
        unsafe{&mut *self.orig}
    }
    pub fn get_tree_elements_mut(&mut self)->PMut<[BBox<N,&mut T>]>{
        //unsafe{&mut *(self.inner.get_elements_mut() as *mut _ as *mut _)}
        unimplemented!();

    }
    pub fn get_tree_elements(&self)->&[BBox<N,&mut T>]{
        unimplemented!();
    }
}


impl<'a,A:Axis,N:Num+'a,T> core::ops::Deref for DinoTreeRefInd<'a,A,N,T>{
    type Target=DinoTree<'a,A,BBox<N,&'a mut T>>;
    fn deref(&self)->&Self::Target{
        //TODO do these in one place
        unsafe{&*(self.inner.as_tree() as *const _ as *const _)}
    }
}
pub struct DinoTreeRef<'a,A:Axis,T:Aabb>{
    inner:DinoTreeInner<A,NodePtr<T>>,
    orig:*mut [T],
    _p:PhantomData<&'a mut T>
}

impl<'a,A:Axis,T:Aabb> core::ops::Deref for DinoTreeRef<'a,A,T>{
    type Target=DinoTree<'a,A,T>;
    fn deref(&self)->&Self::Target{
        //TODO do these in one place
        unsafe{&*(&self.inner as *const _ as *const _)}
    }
}
impl<'a,A:Axis,T:Aabb> core::ops::DerefMut for DinoTreeRef<'a,A,T>{
    fn deref_mut(&mut self)->&mut Self::Target{
        //TODO do these in one place
        unsafe{&mut *(&mut self.inner as *mut _ as *mut _)}
    }
}

impl<'a,T:Aabb> DinoTreeRef<'a,DefaultA,T>{
    pub fn new(arr:&'a mut [T])->DinoTreeRef<'a,DefaultA,T>{
        DinoTreeRef::with_axis(default_axis(),arr)
    }
}

impl<'a,A:Axis,T:Aabb> DinoTreeRef<'a,A,T>{
    pub fn with_axis(a:A,arr:&'a mut [T])->DinoTreeRef<'a,A,T>{
        let inner=make_owned(a,arr);
        let orig=arr as *mut _;
        DinoTreeRef{
            inner,
            orig,
            _p:PhantomData
        }        
    }
    pub fn get_elements(&self)->&[T]{
        unsafe{&*self.orig}
    }
    pub fn get_elements_mut(&mut self)->PMut<'a,[T]>{
        PMut::new(unsafe{&mut *self.orig})
    }
}



///A Node in a dinotree.
pub(crate) struct NodePtr<T: Aabb> {
    _range: PMutPtr<[T]>,

    //range is empty iff cont is none.
    _cont: Option<axgeom::Range<T::Num>>,
    //for non leafs:
    //  div is some iff mid is nonempty.
    //  div is none iff mid is empty.
    //for leafs:
    //  div is none
    _div: Option<T::Num>,
}

pub(crate) fn make_owned<A: Axis, T: Aabb>(axis: A, bots: &mut [T]) -> DinoTreeInner<A, NodePtr<T>> {
    
    let inner = DinoTree::with_axis(axis, bots);
    let inner: Vec<_> = inner
        .inner
        .inner
        .into_nodes()
        .drain(..)
        .map(|mut node| NodePtr {
            _range: node.range.as_ptr(),
            _cont: node.cont,
            _div: node.div,
        })
        .collect();
    let inner = compt::dfs_order::CompleteTreeContainer::from_preorder(inner).unwrap();
    DinoTreeInner {
        axis,
        inner
    }
}

fn make_owned_par<A: Axis, T: Aabb + Send + Sync>(axis: A, bots: &mut [T]) -> DinoTreeInner<A, NodePtr<T>> {
    let inner = DinoTree::with_axis_par(axis, bots);
    let inner: Vec<_> = inner
        .inner
        .inner
        .into_nodes()
        .drain(..)
        .map(|mut node| NodePtr {
            _range: node.range.as_ptr(),
            _cont: node.cont,
            _div: node.div,
        })
        .collect();
    let inner = compt::dfs_order::CompleteTreeContainer::from_preorder(inner).unwrap();
    DinoTreeInner {
        axis,
        inner
    }
}



pub struct DinoTreeOwnedInd<A: Axis,N:Num, T> {
    inner:DinoTreeOwned<A,BBox<N,ThreadPtr<T>>>,
    bots:Vec<T>
}

impl<N:Num,T:Send+Sync> DinoTreeOwnedInd<DefaultA,N,T>{
    pub fn new_par(bots: Vec<T>,func:impl FnMut(&T)->Rect<N>) -> DinoTreeOwnedInd<DefaultA,N, T> {
        DinoTreeOwnedInd::with_axis_par(default_axis(),bots,func)
    }
}
impl<A:Axis,N:Num,T:Send+Sync> DinoTreeOwnedInd<A,N,T>{
    pub fn with_axis_par(axis: A, mut bots: Vec<T>,mut func:impl FnMut(&T)->Rect<N>) -> DinoTreeOwnedInd<A,N, T> {
        let bbox = bots
            .iter_mut()
            .map(|b| BBox::new(func(b), ThreadPtr(b as *mut _) ))
            .collect();
        
        let inner= DinoTreeOwned::with_axis_par(axis,bbox); 
        DinoTreeOwnedInd {
            inner,
            bots,
        }
        
    }
}

impl<N:Num,T> DinoTreeOwnedInd<DefaultA,N, T> {
    pub fn new(bots: Vec<T>,func:impl FnMut(&T)->Rect<N>) -> DinoTreeOwnedInd<DefaultA, N,T> {
        Self::with_axis(default_axis(), bots,func)
    }    
}
impl<A:Axis,N:Num,T> DinoTreeOwnedInd<A,N,T>{
    ///Create an owned dinotree in one thread.
    pub fn with_axis(axis: A, mut bots: Vec<T>,mut func:impl FnMut(&T)->Rect<N>) -> DinoTreeOwnedInd<A,N, T> {
        let bbox = bots
            .iter_mut()
            .map(|b| BBox::new(func(b), ThreadPtr(b as *mut _)))
            .collect();
        
        let inner= DinoTreeOwned::with_axis(axis,bbox); 
        DinoTreeOwnedInd {
            inner,
            bots,
        }
        
    }

    ///Cant use Deref because of lifetime
    pub fn as_tree(&self)->&DinoTree<A,BBox<N,&mut T>>{
        unsafe{&*(self.inner.as_tree() as *const _ as *const _)}
    }

    ///Cant use Deref because of lifetime
    pub fn as_tree_mut(&mut self)->&mut DinoTree<A,BBox<N,&mut T>>{
        unsafe{&mut *(self.inner.as_tree_mut() as *mut _ as *mut _)}
    }

    
    pub fn get_elements(&self) -> &[T] {
        &self.bots
    }
    pub fn get_elements_mut(&mut self) -> &mut [T] {
        &mut self.bots
    }
}



///An owned dinotree componsed of `T:Aabb`
pub struct DinoTreeOwned<A: Axis, T: Aabb> {
    tree: DinoTreeInner<A, NodePtr<T>>,
    bots: Vec<T>,
}

impl<T: Aabb+Send+Sync> DinoTreeOwned<DefaultA, T> {
    pub fn new_par(bots:Vec<T>)->DinoTreeOwned<DefaultA,T>{
        DinoTreeOwned::with_axis_par(default_axis(),bots)
    }
}
impl<A: Axis, T: Aabb+Send+Sync> DinoTreeOwned<A, T> {
    pub fn with_axis_par(axis:A,mut bots:Vec<T>)->DinoTreeOwned<A,T>{
        DinoTreeOwned{
            tree:make_owned_par(axis,&mut bots),
            bots
        }
    }
}
impl<T: Aabb> DinoTreeOwned<DefaultA, T> {
    pub fn new(bots: Vec<T>) -> DinoTreeOwned<DefaultA, T> {
        Self::with_axis(default_axis(), bots)
    }
    
}
impl<A: Axis, T: Aabb> DinoTreeOwned<A, T> {
    ///Create an owned dinotree in one thread.
    pub fn with_axis(axis: A, mut bots: Vec<T>) -> DinoTreeOwned<A, T> {
        DinoTreeOwned {
            tree: make_owned(axis, &mut bots),
            bots,
        }
    }
    
    ///Cant use Deref because of lifetime
    pub fn as_tree(&self)->&DinoTree<A,T>{
        unsafe{&*(&self.tree as *const _ as *const _)}
    }

    ///Cant use Deref because of lifetime
    pub fn as_tree_mut(&mut self)->&mut DinoTree<A,T>{
        unsafe{&mut *(&mut self.tree as *mut _ as *mut _)}
    }

    pub fn get_elements(&self) -> &[T] {
        &self.bots
    }
    pub fn get_elements_mut(&mut self) -> PMut<[T]> {
        PMut::new(&mut self.bots)
    }
    pub fn rebuild(&mut self, mut func: impl FnMut(&mut [T])) {
        func(&mut self.bots);

        let axis = self.tree.axis;
        self.tree = make_owned(axis, &mut self.bots);
    }

}
