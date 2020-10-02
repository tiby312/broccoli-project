use super::*;
use tree::builder::DinoTreeBuilder;

///A version of dinotree where the elements are not sorted along each axis, like a KD Tree.
/// For comparison, a normal kd-tree is provided by `NotSorted`. In this tree, the elements are not sorted
/// along an axis at each level. Construction of `NotSorted` is faster than `DinoTree` since it does not have to
/// sort bots that belong to each node along an axis. But most query algorithms can usually take advantage of this
/// extra property.
pub struct NotSorted<'a,A: Axis, T:Aabb>(pub(crate) DinoTree<'a,A, T>);

impl<'a, T: Aabb + Send + Sync> NotSorted<'a,DefaultA, T> {
    pub fn new_par(bots: &'a mut [T]) -> NotSorted< 'a,DefaultA, T> {
        DinoTreeBuilder::new(bots).build_not_sorted_par()
    }
}
impl<'a, T: Aabb> NotSorted<'a, DefaultA, T> {
    pub fn new(bots: &'a mut [T]) -> NotSorted<'a, DefaultA, T> {
        DinoTreeBuilder::new(bots).build_not_sorted_seq()
    }
}

impl< 'a,A: Axis, T: Aabb + Send + Sync> NotSorted<'a,A,T> {
    pub fn with_axis_par(axis: A, bots: &'a mut [T]) -> NotSorted<'a, A, T> {
        DinoTreeBuilder::with_axis(axis, bots).build_not_sorted_par()
    }
}
impl<'a, A: Axis, T: Aabb> NotSorted<'a, A, T> {
    pub fn with_axis(axis: A, bots: &'a mut [T]) -> NotSorted<'a, A,T> {
        DinoTreeBuilder::with_axis(axis, bots).build_not_sorted_seq()
    }
}

impl<'a,A:Axis,T:Aabb+HasInner> NotSortedQueries<'a> for NotSorted<'a,A,T>{
    type A=A;
    type T=T;
    type Num=T::Num;
    type Inner=T::Inner;

    #[inline(always)]
    fn axis(&self)->Self::A{
        self.0.axis()
    }

    #[inline(always)]
    fn vistr_mut(&mut self)->VistrMut<NodeMut<'a,T>>{
        self.0.vistr_mut()
    }

    #[inline(always)]
    fn vistr(&self)->Vistr<NodeMut<'a,T>>{
        self.0.vistr()
    }
}

impl<'a,A: Axis, T:Aabb> NotSorted< 'a,A, T> {
    #[inline(always)]
    pub fn get_height(&self) -> usize {
        self.0.get_height()
    }
}