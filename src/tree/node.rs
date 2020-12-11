use super::*;

///When we traverse the tree in read-only mode, we can simply return a reference to each node.
///We don't need to protect the user from only mutating parts of the BBox's since they can't
///change anything.
pub type Vistr<'a, N> = compt::dfs_order::Vistr<'a, N, compt::dfs_order::PreOrder>;

mod vistr_mut {
    use crate::inner_prelude::*;

    /// Tree Iterator that returns a protected mutable reference to each node.
    #[repr(transparent)]
    pub struct VistrMut<'a, N> {
        pub(crate) inner: compt::dfs_order::VistrMut<'a, N, compt::dfs_order::PreOrder>,
    }

    impl<'a, N> VistrMut<'a, N> {
        ///It is safe to borrow the iterator and then produce mutable references from that
        ///as long as by the time the borrow ends, all the produced references also go away.
        #[inline(always)]
        pub fn create_wrap_mut(&mut self) -> VistrMut<N> {
            VistrMut {
                inner: self.inner.create_wrap_mut(),
            }
        }

        #[inline(always)]
        pub fn as_slice_mut(&mut self) -> PMut<[N]> {
            PMut::new(self.inner.as_slice_mut())
        }

        #[inline(always)]
        pub fn into_slice(self) -> PMut<'a, [N]> {
            PMut::new(self.inner.into_slice())
        }
    }

    impl<'a, N> core::ops::Deref for VistrMut<'a, N> {
        type Target = Vistr<'a, N>;

        #[inline(always)]
        fn deref(&self) -> &Vistr<'a, N> {
            unsafe { &*(self as *const VistrMut<_> as *const Vistr<_>) }
        }
    }

    unsafe impl<'a, N> compt::FixedDepthVisitor for VistrMut<'a, N> {}

    impl<'a, N> Visitor for VistrMut<'a, N> {
        type Item = PMut<'a, N>;

        #[inline(always)]
        fn next(self) -> (Self::Item, Option<[Self; 2]>) {
            let (nn, rest) = self.inner.next();

            let k = match rest {
                Some([left, right]) => Some([VistrMut { inner: left }, VistrMut { inner: right }]),
                None => None,
            };
            (PMut::new(nn), k)
        }

        #[inline(always)]
        fn level_remaining_hint(&self) -> (usize, Option<usize>) {
            self.inner.level_remaining_hint()
        }

        #[inline(always)]
        fn dfs_preorder(self, mut func: impl FnMut(Self::Item)) {
            self.inner.dfs_preorder(move |a| func(PMut::new(a)));
        }
    }
}
pub use vistr_mut::VistrMut;

///A Node in a Tree.
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
impl<'a, T: Aabb> AsRef<NodePtr<T>> for NodeMut<'a, T> {
    fn as_ref(&self) -> &NodePtr<T> {
        unsafe { &*(self as *const _ as *const _) }
    }
}

///A node in [`Tree`].
pub struct NodeMut<'a, T: Aabb> {
    pub range: PMut<'a, [T]>,
    //range is empty iff cont is none.
    pub cont: Option<axgeom::Range<T::Num>>,
    //for non leafs:
    //  div is some if either this node as bots or a child does
    //for leafs:
    //  div is none
    pub div: Option<T::Num>,
}
