use super::*;


pub(super) struct TreeRefInner<A: Axis, T: Aabb> {
    pub(super) inner: TreeInner<A, NodePtr<T>>,
    pub(super) orig: Ptr<[T]>,
}


impl<A: Axis, T: Aabb + Send + Sync> TreeRefInner<A, T> {
    pub fn with_axis_par(a: A, arr: &mut [T]) -> TreeRefInner<A, T> {
        let inner = make_owned_par(a, arr);
        let orig = Ptr(arr as *mut _);
        TreeRefInner { inner, orig }
    }
}
impl<A: Axis, T: Aabb> TreeRefInner<A, T> {
    pub fn with_axis(a: A, arr: &mut [T]) -> TreeRefInner<A, T> {
        let inner = make_owned(a, arr);
        let orig = Ptr(arr as *mut _);
        TreeRefInner { inner, orig }
    }
}



pub(super) struct TreeIndInner<A:Axis,N:Num,T>{
    pub(super) inner:TreeOwned<A,BBox<N,Ptr<T>>>,
    pub(super) orig:Ptr<[T]>
}

impl<A:Axis,N:Num,T:Send+Sync> TreeIndInner<A,N,T>{
    pub fn with_axis_par(
        axis: A,
        arr: &mut [T],
        mut func: impl FnMut(&mut T) -> Rect<N>,
    ) -> TreeIndInner<A, N, T> {
        let orig = Ptr(arr as *mut _);
        let bbox = arr
            .iter_mut()
            .map(|b| BBox::new(func(b), Ptr(b as *mut _)))
            .collect();

        let inner = TreeOwned::with_axis_par(axis, bbox);

        TreeIndInner{
            inner,
            orig
        }
    }
}
impl<A:Axis,N:Num,T> TreeIndInner<A,N,T>{
    pub fn with_axis(
        axis: A,
        arr: &mut [T],
        mut func: impl FnMut(&mut T) -> Rect<N>,
    ) -> TreeIndInner<A, N, T> {
        let orig = Ptr(arr as *mut _);
        let bbox = arr
            .iter_mut()
            .map(|b| BBox::new(func(b), Ptr(b as *mut _)))
            .collect();

        let inner = TreeOwned::with_axis(axis, bbox);

        TreeIndInner{
            inner,
            orig
        }
    }
}



pub(super) fn make_owned<A: Axis, T: Aabb>(axis: A, bots: &mut [T]) -> TreeInner<A, NodePtr<T>> {
    let inner = crate::with_axis(axis, bots);

    
    let inner: compt::dfs_order::CompleteTreeContainer<NodePtr<T>,_> = inner.inner.inner.convert();

    TreeInner { axis, inner }
}


fn make_owned_par<A: Axis, T: Aabb + Send + Sync>(
    axis: A,
    bots: &mut [T],
) -> TreeInner<A, NodePtr<T>> {
    let inner = crate::with_axis_par(axis, bots);

    let inner: compt::dfs_order::CompleteTreeContainer<NodePtr<T>,_> = inner.inner.inner.convert();    

    TreeInner { axis, inner }
}
