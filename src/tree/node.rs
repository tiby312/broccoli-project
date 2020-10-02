use super::*;

///When we traverse the tree in read-only mode, we can simply return a reference to each node.
///We don't need to protect the user from only mutating parts of the BBox's since they can't
///change anything.
pub type Vistr<'a, N> = compt::dfs_order::Vistr<'a, N, compt::dfs_order::PreOrder>;

mod vistr_mut {
    use crate::inner_prelude::*;

    //Cannot use since we need create_wrap_mut()
    //We must create our own new type.
    //pub type VistrMut<'a,N> = compt::MapStruct<compt::dfs_order::VistrMut<'a,N,compt::dfs_order::PreOrder>,Foo<'a,N>>;

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
        pub fn into_slice(self) -> PMut<'a,[N]> {
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
                Some([left, right]) => {
                    Some([VistrMut { inner: left }, VistrMut { inner: right }])
                }
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


///Expose a node trait api to hide the lifetime of NodeMut.
///This way query algorithms do not need to worry about this lifetime.
pub trait Node {
    type T: Aabb<Num = Self::Num>;
    type Num: Num;
    fn get(&self) -> NodeRef<Self::T>;
    fn get_mut(&mut self) -> NodeRefMut<Self::T>;
}

impl<'a, T: Aabb> Node for NodeMut<'a, T> {
    type T = T;
    type Num = T::Num;
    fn get(&self) -> NodeRef<Self::T> {
        //TODO point as struct impl
        NodeRef {
            bots: self.range.as_ref(),
            cont: &self.cont,
            div: &self.div,
        }
    }
    fn get_mut(&mut self) -> NodeRefMut<Self::T> {
        NodeRefMut {
            bots: self.range.as_mut(),
            cont: &self.cont,
            div: &self.div,
        }
    }
}

///A lifetimed node in a dinotree.
pub struct NodeMut<'a, T: Aabb> {
    pub(crate) range: PMut<'a, [T]>,

    //range is empty iff cont is none.
    pub(crate) cont: Option<axgeom::Range<T::Num>>,
    //for non leafs:
    //  div is some iff mid is nonempty.
    //  div is none iff mid is empty.
    //for leafs:
    //  div is none
    pub(crate) div: Option<T::Num>,
}

impl<'a, T: Aabb> NodeMut<'a, T> {
    pub fn get(&self) -> NodeRef<T> {
        NodeRef {
            bots: self.range.as_ref(),
            cont: &self.cont,
            div: &self.div,
        }
    }
    pub fn get_mut(&mut self) -> NodeRefMut<T> {
        NodeRefMut {
            bots: self.range.as_mut(),
            cont: &self.cont,
            div: &self.div,
        }
    }
}

///Mutable reference to a node in the dinotree.
pub struct NodeRefMut<'a, T: Aabb> {
    ///The bots that belong to this node.
    pub bots: PMut<'a, [T]>,

    ///Is None iff bots is empty.
    pub cont: &'a Option<axgeom::Range<T::Num>>,

    ///Is None if node is a leaf, or there are no bots in this node or in any decendants.
    pub div: &'a Option<T::Num>,
}

///Reference to a node in the dinotree.
pub struct NodeRef<'a, T: Aabb> {
    ///The bots that belong to this node.
    pub bots: &'a [T],

    ///Is None iff bots is empty.
    pub cont: &'a Option<axgeom::Range<T::Num>>,

    ///Is None if node is a leaf, or there are no bots in this node or in any decendants.
    pub div: &'a Option<T::Num>,
}
