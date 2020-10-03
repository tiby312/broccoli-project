use crate::inner_prelude::*;

#[cfg(test)]
mod tests;


pub mod assert;

pub mod analyze;

///Contains code to write generic code that can be run in parallel, or sequentially. The api is exposed
///in case users find it useful when writing parallel query code to operate on the tree.
pub mod par;

mod notsorted;
pub(crate) use self::notsorted::NotSorted;


pub mod builder;
use builder::TreeBuilder;

pub(crate) use self::node::*;

///Contains node-level building block structs and visitors used for a Tree.
pub mod node;


pub mod collections;

use crate::query::*;



pub(crate) struct TreeInner<A: Axis, N> {
    axis: A,
    inner: compt::dfs_order::CompleteTreeContainer<N, compt::dfs_order::PreOrder>
}

///The data structure this crate revoles around.
#[repr(transparent)]
pub struct Tree<'a,A: Axis, T:Aabb> {
    inner:TreeInner<A,NodeMut<'a,T>>
}

///The type of the axis of the first node in the Tree.
///If it is the y axis, then the first divider will be a horizontal line,
///since it is partioning space based off of objects y value.
pub type DefaultA = YAXIS;
///Constructor of the default axis type. Needed since you cannot construct from type alias's.
pub const fn default_axis() -> YAXIS {
    YAXIS
}

/// # Examples
///
///```
///let mut bots = [axgeom::rect(0,10,0,10)];
///let tree = broccoli::new(&mut bots);
///
///```
pub fn new<'a,T:Aabb>(bots:&'a mut [T])->Tree<'a,DefaultA,T>{
    TreeBuilder::new(bots).build_seq()
}


/// # Examples
///
///```
///let mut bots = [axgeom::rect(0,10,0,10)];
///let tree = broccoli::with_axis(axgeom::XAXIS,&mut bots);
///
///```
pub fn with_axis<'a,A:Axis,T:Aabb>(axis:A,bots:&'a mut [T])->Tree<'a,A,T>{
    TreeBuilder::with_axis(axis, bots).build_seq()
}


/// # Examples
///
///```
///let mut bots = [axgeom::rect(0,10,0,10)];
///let tree = broccoli::new_par(&mut bots);
///
///```
pub fn new_par<'a,T:Aabb+Send+Sync>(bots:&'a mut [T])->Tree<'a,DefaultA,T>{
    TreeBuilder::new(bots).build_par()
}


/// # Examples
///
///```
///let mut bots = [axgeom::rect(0,10,0,10)];
///let tree = broccoli::with_axis_par(axgeom::XAXIS,&mut bots);
///
///```
pub fn with_axis_par<'a,A:Axis,T:Aabb+Send+Sync>(axis:A,bots:&'a mut [T])->Tree<'a,A,T>{
    TreeBuilder::with_axis(axis, bots).build_par()
}

impl<'a,A:Axis,T:Aabb+HasInner> QueriesInner<'a> for Tree<'a,A,T>{
    type Inner=T::Inner;
}

impl<'a,A:Axis,T:Aabb> Queries<'a> for Tree<'a,A,T>{
    type A=A;
    type T=T;
    type Num=T::Num;
    
    #[inline(always)]
    fn axis(&self)->Self::A{
        self.inner.axis
    }

    #[inline(always)]
    fn vistr_mut(&mut self)->VistrMut<NodeMut<'a,T>>{
        VistrMut{inner:self.inner.inner.vistr_mut()}
    }

    #[inline(always)]
    fn vistr(&self)->Vistr<NodeMut<'a,T>>{
        self.inner.inner.vistr()
    }
}


pub struct CollidingPairs<'a, T, D> {
    ///See collect_intersections_list()
    ///The same elements can be part of
    ///multiple intersecting pairs.
    ///So pointer aliasing rules are not
    ///being met if we were to just use this
    ///vec according to its type signature.
    cols: Vec<(*mut T, *mut T, D)>,
    _p:PhantomData<&'a mut T>
}
impl<'a,T,D> CollidingPairs<'a,T,D>{
    pub fn for_every_pair_mut<'b, A: Axis, N: Num>(
        &'b mut self,
        mut func: impl FnMut(&mut T, &mut T, &mut D),
    ) {
        for (a, b, d) in self.cols.iter_mut() {
            func(unsafe{&mut **a}, unsafe{&mut **b}, d)
        }
    }
}



struct Ptr<T>(*mut T);
unsafe impl<T> Send for Ptr<T>{}
unsafe impl<T> Sync for Ptr<T>{}

///All colliding pairs partitioned into
///mutually exclusive sets so that they can
//be traversed in parallel
pub struct CollidingPairsPar<'a,T:Send+Sync,D:Send+Sync>{
    cols: Vec<Vec<(Ptr<T>, Ptr<T>, D)>>,
    _p:PhantomData<&'a mut T>
}
impl<'a,T:Send+Sync,D:Send+Sync> CollidingPairsPar<'a,T,D>{
    pub fn for_every_pair_mut_par<A: Axis, N: Num>(
        &mut self,
        func: impl Fn(&mut T, &mut T, &mut D) + Send + Sync + Copy,
    ) {
        
        fn parallelize<T: Visitor + Send + Sync>(a: T, func: impl Fn(T::Item) + Sync + Send + Copy)
        where
            T::Item: Send + Sync,
        {
            let (n, l) = a.next();
            func(n);
            if let Some([left, right]) = l {
                rayon::join(|| parallelize(left, func), || parallelize(right, func));
            }
        }
        let mtree = compt::dfs_order::CompleteTree::from_preorder_mut(&mut self.cols).unwrap();

        parallelize(mtree.vistr_mut(), |a| {
            for (a, b, d) in a.iter_mut() {
                let a = unsafe{&mut *a.0};
                let b = unsafe{&mut *b.0};
                func(a, b, d)
            }
        });
    }
}
impl<'a,'b,A:Axis,N:Num,T:Send+Sync> Tree<'a,A,BBox<N,&'b mut T>>{
    pub fn collect_colliding_pairs_par<D: Send + Sync>(
        &mut self,
         func: impl Fn(&mut T, &mut T) -> Option<D> + Send + Sync+Copy,
    ) -> CollidingPairsPar<T,D>{
        let cols = self.collect_colliding_pairs_par_inner(|a, b| {
            match func(a, b) {
                Some(d) => Some((Ptr(a as *mut _), Ptr(b as *mut _), d)),
                None => None,
            }
        });
        CollidingPairsPar{
            cols,
            _p: PhantomData,
        }
    }
    
    fn collect_colliding_pairs_par_inner<D: Send + Sync>(
        &mut self,
        func: impl Fn(&mut T, &mut T) -> Option<D> + Send + Sync+Copy,
    ) -> Vec<Vec<D>> {
        

        struct Foo<T: Visitor> {
            current: T::Item,
            next: Option<[T; 2]>,
        }
        impl<T: Visitor> Foo<T> {
            fn new(a: T) -> Foo<T> {
                let (n, f) = a.next();
                Foo {
                    current: n,
                    next: f,
                }
            }
        }
    
        //TODO might break if user uses custom height
        let height =
            1 + par::compute_level_switch_sequential(par::SWITCH_SEQUENTIAL_DEFAULT, self.get_height())
                .get_depth_to_switch_at();
        //dbg!(tree.get_height(),height);
        let mut cols: Vec<Vec<D>> = (0..compt::compute_num_nodes(height))
            .map(|_| Vec::new())
            .collect();
        let mtree = compt::dfs_order::CompleteTree::from_preorder_mut(&mut cols).unwrap();
    
        self.find_colliding_pairs_par_ext(
            move |a| {
                let next = a.next.take();
                if let Some([left, right]) = next {
                    let l = Foo::new(left);
                    let r = Foo::new(right);
                    *a = l;
                    r
                } else {
                    unreachable!()
                }
            },
            move |_a, _b| {},
            move |c, a, b| {
                if let Some(d) = func(a, b) {
                    c.current.push(d);
                }
            },
            Foo::new(mtree.vistr_mut()),
        );

        cols
        //CollidingPairsPar{cols,_p:PhantomData}
    
    }
}

impl<'a,'b,A:Axis,N:Num,T> Tree<'a,A,BBox<N,&'b mut T>>{
    pub fn collect_colliding_pairs<'c,D: Send + Sync>(
        &mut self,
        mut func: impl FnMut(&mut T, &mut T) -> Option<D> + Send + Sync,
    ) -> CollidingPairs<'b, T, D> {
        let mut cols: Vec<_> = Vec::new();
    
        self.find_colliding_pairs_mut(|a, b| {
            if let Some(d) = func(a, b) {
                //We use unsafe to collect mutable references of
                //all colliding pairs.
                //This is safe to do because the user is forced
                //to iterate through all the colliding pairs
                //one at a time.
                let a=*a as *mut T;
                let b=*b as *mut T;
                
                cols.push((a,b,d));
            }
        });

        CollidingPairs {
            cols,
            _p:PhantomData
        }
    }
}



impl<'a, A: Axis, T:Aabb> Tree<'a, A,T> {

    /// # Examples
    ///
    ///```
    ///use broccoli::analyze;
    ///const NUM_ELEMENT:usize=400;
    ///let mut bots = [axgeom::rect(0,10,0,10);NUM_ELEMENT];
    ///let mut tree = broccoli::new(&mut bots);
    ///
    ///assert_eq!(tree.get_height(),analyze::compute_tree_height_heuristic(NUM_ELEMENT,analyze::DEFAULT_NUMBER_ELEM_PER_NODE));
    ///```
    ///
    #[must_use]
    #[inline(always)]
    pub fn get_height(&self) -> usize {
        self.inner.inner.get_height()
    }

    

    /// # Examples
    ///
    ///```
    ///use broccoli::analyze;
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let mut tree = broccoli::new(&mut bots);
    ///
    ///assert_eq!(tree.num_nodes(),analyze::nodes_left(0,tree.get_height() ));
    ///
    ///```
    #[must_use]
    #[inline(always)]
    pub fn num_nodes(&self) -> usize {
        self.inner.inner.get_nodes().len()
    }
}

