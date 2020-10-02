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

impl<'a,'b,A:Axis,N:Num,T> Tree<'a,A,BBox<N,&'b mut T>>{
    pub fn collect_intersections_list<'c,D: Send + Sync>(
        &mut self,
        mut func: impl FnMut(&mut T, &mut T) -> Option<D> + Send + Sync,
    ) -> CollidingPairs<'b, T, D> {
        let mut cols: Vec<_> = Vec::new();
    
        self.find_intersections_mut(|a, b| {
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
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let mut tree = broccoli::new(&mut bots);
    ///
    ///assert_eq!(tree.get_height(),analyze::compute_tree_height_heuristic(400,analyze::DEFAULT_NUMBER_ELEM_PER_NODE));
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

