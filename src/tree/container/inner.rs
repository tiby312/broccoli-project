use super::*;

pub(super) struct TreeIndInner<N: Num, T> {
    pub(super) inner: TreeOwned<BBox<N, Ptr<T>>>,
    pub(super) orig: Ptr<[T]>,
}

impl<N: Num + Send + Sync, T: Send + Sync> TreeIndInner<N, T> {
    pub fn new_par(arr: &mut [T], mut func: impl FnMut(&mut T) -> Rect<N>) -> TreeIndInner<N, T> {
        let orig = Ptr(arr as *mut _);
        let bbox = arr
            .iter_mut()
            .map(|b| BBox::new(func(b), Ptr(b as *mut _)))
            .collect();

        let inner = TreeOwned::new_par(bbox);

        TreeIndInner { inner, orig }
    }
}
impl<N: Num, T> TreeIndInner<N, T> {
    pub fn new(arr: &mut [T], mut func: impl FnMut(&mut T) -> Rect<N>) -> TreeIndInner<N, T> {
        let orig = Ptr(arr as *mut _);
        let bbox = arr
            .iter_mut()
            .map(|b| BBox::new(func(b), Ptr(b as *mut _)))
            .collect();

        let inner = TreeOwned::new(bbox);

        TreeIndInner { inner, orig }
    }
}

pub(super) fn make_owned<T: Aabb>(bots: &mut [T]) -> TreePtr<T> {
    let tree = crate::new(bots);

    TreePtr{
        _inner:tree.inner.convert(),
        _num_aabbs:tree.num_aabbs
    }
    
}

pub(super) fn make_owned_par<T: Aabb + Send + Sync>(bots: &mut [T]) -> TreePtr<T>
where
    T::Num: Send + Sync,
{
    let tree = crate::new_par(bots);

    TreePtr{
        _inner:tree.inner.convert(),
        _num_aabbs:tree.num_aabbs
    }
}
