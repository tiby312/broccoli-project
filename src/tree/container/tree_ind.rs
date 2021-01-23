use super::*;



///This is a `Vec<BBox<N,&'a mut T>>` under the hood
///with the added guarentee that all the `&'a mut T`
///point to the same slice.
///
///From this struct a user can create a [`TreeInd`].
pub struct TreeIndBase<'a,N:Num,T>{
    aabbs:Box<[BBox<N,&'a mut T>]>,
    orig:Ptr<[T]>
}
impl<'a,N:Num,T> TreeIndBase<'a,N,T>{

    /// Create a [`TreeIndBase`].
    ///
    /// # Examples
    ///
    ///```
    /// let mut aabbs = [
    ///    broccoli::bbox(broccoli::rect(0isize, 10, 0, 10), 0),
    /// ];
    ///
    /// let mut base=broccoli::container::TreeIndBase::new(&mut aabbs,|a|a.rect); 
    /// let mut tree = base.build();
    /// ```
    pub fn new(bots:&'a mut [T],mut func:impl FnMut(&mut T)->Rect<N>)->TreeIndBase<'a,N,T>{
        let orig = Ptr(bots as *mut _);

        TreeIndBase{
            orig,
            aabbs:bots.iter_mut().map(|a|crate::bbox(func(a),a)).collect::<Vec<_>>().into_boxed_slice()
        }
    }

    /// Extra the internals of a [`TreeIndBase`].
    ///
    /// # Examples
    ///
    ///```
    /// let mut aabbs = [
    ///    broccoli::bbox(broccoli::rect(0isize, 10, 0, 10), 0),
    /// ];
    ///
    /// let mut base=broccoli::container::TreeIndBase::new(&mut aabbs,|a|a.rect); 
    /// let mut inner=base.into_inner();
    /// let mut tree = broccoli::new(&mut inner);
    /// //We can make a tree using the internals, but we lost the guarentee
    /// //that all the `&'a mut T` belong to the same slice.
    /// ```
    pub fn into_inner(self)->Box<[BBox<N,&'a mut T>]>{
        self.aabbs
    }

    /// Build a [`TreeInd`].
    ///
    /// # Examples
    ///
    ///```
    /// let mut aabbs = [
    ///    broccoli::bbox(broccoli::rect(0isize, 10, 0, 10), 0),
    /// ];
    ///
    /// let mut base=broccoli::container::TreeIndBase::new(&mut aabbs,|a|a.rect); 
    /// let mut tree = base.build();
    /// ```
    pub fn build<'b>(&'b mut self)->TreeInd<'a,'b,N,T>{
        let tree=crate::new(&mut self.aabbs);

        TreeInd{
            tree,
            orig:self.orig
        }
    }

    /// Build a [`TreeInd`].
    ///
    /// # Examples
    ///
    ///```
    /// let mut aabbs = [
    ///    broccoli::bbox(broccoli::rect(0isize, 10, 0, 10), 0),
    /// ];
    ///
    /// let mut base=broccoli::container::TreeIndBase::new(&mut aabbs,|a|a.rect); 
    /// let mut tree = base.build_par(broccoli::RayonJoin);
    /// ```
    pub fn build_par<'b>(&'b mut self,joiner:impl crate::Joinable)->TreeInd<'a,'b,N,T> where N:Send+Sync,T:Send+Sync{
        let tree=crate::new_par(joiner,&mut self.aabbs);

        TreeInd{
            tree,
            orig:self.orig
        }
    }

}



/// A less general tree that provides `collect` functions
/// and also derefs to a [`Tree`].
///
/// [`TreeInd`] assumes there is a layer of indirection where
/// all the pointers point to the same slice.
/// It uses this assumption to provide `collect` functions that allow
/// storing query results that can then be iterated through multiple times
/// quickly.
///
#[repr(C)]
pub struct TreeInd<'a,'b,N:Num,T>{
    tree:Tree<'b,BBox<N,&'a mut T>>,
    orig:Ptr<[T]>
}

#[repr(C)]
pub(super) struct TreeIndPtr<N:Num,T>{
    pub(super) tree:TreePtr<BBox<N,Ptr<T>>>,
    pub(super) orig:Ptr<[T]>
}


impl<'a,'b, N:Num,T> From<TreeInd<'a,'b,N,T>> for Tree<'b, BBox<N,&'a mut T>> {
    #[inline(always)]
    fn from(a: TreeInd<'a,'b,N,T>) -> Self {
        a.tree
    }
}


impl<'a,'b, N: Num , T> core::ops::Deref for TreeInd<'a,'b, N, T> {
    type Target = Tree<'b, BBox<N, &'a mut T>>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}
impl<'a, 'b,N: Num, T> core::ops::DerefMut for TreeInd<'a, 'b,N, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tree
    }
}

unsafe impl<'a,'b,N:Num,T> FromSlice<'a,'b> for TreeInd<'a,'b,N,T>{
    type T=BBox<N,&'a mut T>;
    type Inner=T;
    type Num=N;
    fn get_inner_elements(&self)->&[Self::Inner]{
        unsafe{&*self.orig.0}
    }

    fn get_inner_elements_mut(&mut self)->&mut [Self::Inner]{
        unsafe{&mut *self.orig.0}
    }

    fn get_tree_mut(&mut self)->&mut Tree<'b,BBox<N,&'a mut T>>{
        self
    }
}




impl<'a,'b,N:Num,T> TreeInd<'a,'b,N,T>{


    pub(super) fn into_ptr(self)->TreeIndPtr<N,T>{
        
        TreeIndPtr{
            tree:TreePtr{
                _inner:unsafe{self.tree.inner.convert()},
                _num_aabbs:self.tree.num_aabbs
            },
            orig:self.orig
        }
    }
}

