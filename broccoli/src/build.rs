//!
//! Tree building blocks
//!

use super::*;

///The default starting axis of a [`Tree`]. It is set to be the `X` axis.
///This means that the first divider is a 'vertical' line since it is
///partitioning space based off of the aabb's `X` value.
#[must_use]
pub const fn default_axis() -> axgeom::XAXIS {
    axgeom::XAXIS
}

///Expose a common Sorter trait so that we may have two version of the tree
///where one implementation actually does sort the tree, while the other one
///does nothing when sort() is called.
pub trait Sorter<T> {
    fn sort(&self, axis: impl Axis, bots: &mut [T]);
}

///Sorts the bots based on an axis.
#[inline(always)]
pub fn sweeper_update<I: Aabb, A: Axis>(axis: A, collision_botids: &mut [I]) {
    collision_botids.sort_unstable_by(|a, b| queries::cmp_aabb(axis, a, b));
}

#[must_use]
pub struct NodeFinisher<'a, T: Aabb> {
    axis: AxisDyn,
    div: Option<T::Num>, //This can be null if there are no bots left at all
    mid: &'a mut [T],
    middle_left_len: Option<usize>,
    min_elem: usize,
    num_elem: usize,
}
impl<'a, T: Aabb> NodeFinisher<'a, T> {
    pub fn get_min_elem(&self) -> usize {
        self.min_elem
    }

    pub fn get_num_elem(&self) -> usize {
        self.num_elem
    }
    #[inline(always)]
    #[must_use]
    pub fn finish<S: Sorter<T>>(self, sorter: &mut S) -> Node<'a, T, T::Num> {
        fn create_cont<A: Axis, T: Aabb>(
            axis: A,
            middle: &[T],
            ml: Option<usize>,
        ) -> axgeom::Range<T::Num> {
            let ml = if let Some(ml) = ml { ml } else { middle.len() };

            match middle.split_first() {
                Some((first, rest)) => {
                    let start = {
                        let mut min = first.range(axis).start;

                        for a in rest[0..ml - 1].iter() {
                            let start = a.range(axis).start;

                            if start < min {
                                min = start;
                            }
                        }
                        min
                    };

                    let end = {
                        let mut max = first.range(axis).end;

                        //The bots to the right of the divier
                        //are more likely to  contain the max
                        //rightmost aabb edge.
                        for a in rest.iter().rev() {
                            let k = a.range(axis).end;

                            if k > max {
                                max = k;
                            }
                        }
                        max
                    };

                    axgeom::Range { start, end }
                }
                None => axgeom::Range {
                    start: Default::default(),
                    end: Default::default(),
                },
            }
        }

        let cont = match self.axis {
            AxisDyn::X => {
                //Create cont first otherwise middle_left_len is no longer valid.
                let c = create_cont(axgeom::XAXIS, self.mid, self.middle_left_len);
                sorter.sort(axgeom::XAXIS.next(), self.mid);
                c
            }
            AxisDyn::Y => {
                let c = create_cont(axgeom::YAXIS, self.mid, self.middle_left_len);
                sorter.sort(axgeom::YAXIS.next(), self.mid);
                c
            }
        };

        Node {
            cont,
            min_elem: self.min_elem,
            //num_elem: self.num_elem,
            range: AabbPin::new(self.mid),
            div: self.div,
        }
    }
}

///
/// The main primitive to build a tree.
///
pub struct TreeBuildVisitor<'a, T> {
    bots: &'a mut [T],
    current_height: usize,
    axis: AxisDyn,
}


use std::hash::Hash;
use std::cmp::Eq;
use std::collections::HashMap;




pub struct TreeEmbryo<N,T>{
    rects:Vec<Rect<N>>,
    map:HashMap<Rect<N>,Vec<T>>,
    num_levels:usize
}

pub struct TreView<'a,N,T>{
    nodes:Vec<Node<'a,T,N>>
}


// The actual tree!!!!
pub struct Tre<N,T>{
    rects:Vec<Rect<N>>,
    values:Vec<T>,
    nodes:Vec<NodeEmbryo<N>>
}
impl<N,T> Tre<N,T>{
    pub fn view(&mut self)->TreView<N,T>{
        todo!()
    }
}



#[test]
fn test(){
    let mut k=[0usize];

    let mut emb=TreeEmbryo::new(5,&mut k,|a|axgeom::Rect::new(0,0,0,0));
    let (visitor,mut acc)=emb.visitors();
    acc.recurse_seq(visitor,&mut DefaultSorter);
    let tree=acc.finish(emb);


}
pub struct Accum<N>{
    target_num_nodes:usize,
    inner:Vec<NodeEmbryo<N>>
}

impl<N:Num+Hash+Eq> Accum<N>{
    pub fn finish<T>(self,mut embryo:TreeEmbryo<N,T>)->Tre<N,T>{
        //TOD add assertion that target num nodes reached.
        let mut values=vec!();

        let mut new_rects=vec!();
        for a in embryo.rects.iter(){
            let j=embryo.map.remove(a).unwrap();
            for c in j.iter(){
                new_rects.push(*a);
            }
            values.extend(j);
        }
        //TODO need to update node's num elem with duplicates
        assert_eq!(new_rects.len(),values.len());
        Tre { rects:new_rects, values,nodes:self.inner }
    }
    pub fn recurse_seq<S: Sorter<Rect<N>>>(&mut self, vis:Vis1<N>,sorter: &mut S, ) {
        let (node,rest) = vis.build_and_next();
        self.add(node.finish(sorter));
        if let Some([left, right]) = rest {
            self.recurse_seq(left,sorter);
            self.recurse_seq(right,sorter);
        }
    }
    fn add(&mut self,a:NodeEmbryo<N>){
        self.inner.push(a);
    }
}


impl<N:Hash+Eq,T> TreeEmbryo<N,T>{

    pub fn new(num_levels:usize,bots:&mut [T],mut func:impl FnMut(&mut T)-> Rect<N>)->TreeEmbryo<N,&mut T>{
        //Got the rects
        let mut map:HashMap<_,Vec<_>>=std::collections::HashMap::new();

        let rects:Vec<_> = bots.iter_mut().map(|a|{
            let r=func(a);
            
            if let Some(z)=map.get_mut(&r){
                z.push(a);
            }else{
                map.insert(r, vec!(a));
            }
            r
        }).collect();

        TreeEmbryo{num_levels,rects,map}
    }

    pub fn visitors(&mut self)->(Vis1<N>,Accum<N>){
        (Vis1{
            rects:&mut self.rects,
            current_height:self.num_levels-1,
            axis:default_axis().to_dyn()
        },Accum{inner:vec!(),target_num_nodes:0}) //TODO right number
    }
    
}


pub struct NodeEmbryo<N>{

    /// May or may not be sorted.
    pub num_elem:usize,

    /// if range is empty, then value is `[default,default]`.
    /// if range is not empty, then cont is the min max bounds in on the y axis (if the node belongs to the x axis).
    pub cont: axgeom::Range<N>,

    /// for non leafs:
    ///   if there is a bot either in this node or in a child node, then div is some.
    ///
    /// for leafs:
    ///   value is none
    pub div: Option<N>,

    // ///
    // /// The minimum number of elements in a child node.
    // /// If the left child has 500 bots, and the right child has 20, then
    // /// this value will be 20.
    // ///
    // /// This is used to determine when to start a parallel task.
    // /// Starting a parallel task has overhead so we only want to split
    // /// one off if we know that both threads have a decent amount of work
    // /// to perform in parallel.
    // ///
    // TODO Is it ok this only counts distinct aabbs? Guess so because cost of dups is prob small?
    pub min_elem: usize,
    
}

pub struct NodeFinish2<'a,N>{
    div:Option<N>,
    axis:AxisDyn,
    min_elem:usize,
    num_elem:usize,
    middle_left_len:Option<usize>,
    mid:&'a mut [Rect<N>]
}
impl<'a,N:Num> NodeFinish2<'a,N>{
    pub fn get_min_elem(&self) -> usize {
        self.min_elem
    }

    pub fn get_num_elem(&self) -> usize {
        self.num_elem
    }
    #[inline(always)]
    #[must_use]
    pub fn finish<S: Sorter<Rect<N>>>(self, sorter: &mut S) -> NodeEmbryo< N> {
        fn create_cont<A: Axis, N: Num>(
            axis: A,
            middle: &mut [Rect<N>],
            ml: Option<usize>,
        ) -> axgeom::Range<N> {
            let ml = if let Some(ml) = ml { ml } else { middle.len() };

            match middle.split_first() {
                Some((first, rest)) => {
                    let start = {
                        let mut min = first.range(axis).start;

                        for a in rest[0..ml - 1].iter() {
                            let start = a.range(axis).start;

                            if start < min {
                                min = start;
                            }
                        }
                        min
                    };

                    let end = {
                        let mut max = first.range(axis).end;

                        //The bots to the right of the divier
                        //are more likely to  contain the max
                        //rightmost aabb edge.
                        for a in rest.iter().rev() {
                            let k = a.range(axis).end;

                            if k > max {
                                max = k;
                            }
                        }
                        max
                    };

                    axgeom::Range { start, end }
                }
                None => axgeom::Range {
                    start: Default::default(),
                    end: Default::default(),
                },
            }
        }

        let cont = match self.axis {
            AxisDyn::X => {
                //Create cont first otherwise middle_left_len is no longer valid.
                let c = create_cont(axgeom::XAXIS, self.mid, self.middle_left_len);
                sorter.sort(axgeom::XAXIS.next(), self.mid);
                c
            }
            AxisDyn::Y => {
                let c = create_cont(axgeom::YAXIS, self.mid, self.middle_left_len);
                sorter.sort(axgeom::YAXIS.next(), self.mid);
                c
            }
        };

        NodeEmbryo {
            cont,
            min_elem: self.min_elem,
            //num_elem: self.num_elem,
            num_elem: self.mid.len(),
            div: self.div,
        }
    }

}

pub struct Vis1<'a,N>{
    rects:&'a mut [Rect<N>],
    current_height:usize,
    axis:AxisDyn
}
impl<'a,N:Num> Vis1<'a,N>{


    pub fn build_and_next(self) -> (NodeFinish2<'a, N>,Option<[Vis1<'a,N>;2]>) {
        //leaf case
        if self.current_height == 0 {
            let node = NodeFinish2 {
                middle_left_len: None,
                mid: self.rects,
                div: None,
                axis: self.axis,
                min_elem: 0,
                num_elem: 0,
            };
            (node,None)
        } else {
            fn construct_non_leaf<N:Num>(
                div_axis: impl Axis,
                bots: &mut [Rect<N>],
            ) -> (NodeFinish2<N>, &mut [Rect<N>], &mut [Rect<N>]) {
                if bots.is_empty() {
                    return (
                        NodeFinish2 {
                            middle_left_len: None,
                            mid: bots,
                            div: None,
                            axis: div_axis.to_dyn(),
                            min_elem: 0,
                            num_elem: 0,
                        },
                        &mut [],
                        &mut [],
                    );
                }

                let med_index = bots.len() / 2;

                let (ll, med, rr) = bots.select_nth_unstable_by(med_index, move |a, b| {
                    crate::queries::cmp_aabb(div_axis, a, b)
                });

                let med_val = med.range(div_axis).start;

                let (ml, ll) = {
                    let mut m = 0;
                    for a in 0..ll.len() {
                        if ll[a].range(div_axis).end >= med_val {
                            //keep
                            ll.swap(a, m);
                            m += 1;
                        }
                    }
                    ll.split_at_mut(m)
                };

                let (mr, rr) = {
                    let mut m = 0;
                    for a in 0..rr.len() {
                        if rr[a].range(div_axis).start <= med_val {
                            //keep
                            rr.swap(a, m);
                            m += 1;
                        }
                    }
                    rr.split_at_mut(m)
                };

                let ml_len = ml.len();
                let ll_len = ll.len();
                let rr_len = rr.len();

                let mr_len = mr.len();

                {
                    let (_, rest) = bots.split_at_mut(ml_len);
                    let (ll, rest) = rest.split_at_mut(ll_len);
                    let (mr2, _) = rest.split_at_mut(1 + mr_len);

                    let (a, b) = if mr2.len() < ll.len() {
                        let (a, _) = ll.split_at_mut(mr2.len());
                        (a, mr2)
                    } else {
                        let (_, a) = mr2.split_at_mut(mr2.len() - ll.len());
                        (a, ll)
                    };
                    a.swap_with_slice(b);
                }

                let left_len = ll_len;
                let right_len = rr_len;
                let mid_len = ml_len + 1 + mr_len;

                let middle_left_len = Some(ml_len + 1);

                let (mid, rest) = bots.split_at_mut(mid_len);
                let (left, right) = rest.split_at_mut(left_len);

                (
                    NodeFinish2 {
                        middle_left_len,
                        mid,
                        div: Some(med_val),
                        axis: div_axis.to_dyn(),
                        min_elem: left_len.min(right_len),
                        num_elem: left_len + right_len,
                    },
                    left,
                    right,
                )
            }

            let (finish_node, left, right) = match self.axis {
                AxisDyn::X => construct_non_leaf(axgeom::XAXIS, self.rects),
                AxisDyn::Y => construct_non_leaf(axgeom::YAXIS, self.rects),
            };

            let current_height= self.current_height.saturating_sub(1);
            let axis=self.axis.next();
            (finish_node,Some([
                Vis1{rects:left,axis,current_height},
                Vis1{rects:right,axis,current_height}
            ]))
            
        }

    }
}


pub struct NodeBuildResult<'a, T: Aabb> {
    pub node: NodeFinisher<'a, T>,
    pub rest: Option<[TreeBuildVisitor<'a, T>; 2]>,
}

impl<'a, T: Aabb + ManySwap> TreeBuildVisitor<'a, T> where T::Num:std::hash::Hash+std::cmp::Eq{
    pub fn get_bots(&self) -> &[T] {
        self.bots
    }
    #[must_use]
    pub fn new(num_levels: usize, bots: &'a mut [T]) -> TreeBuildVisitor<'a, T> {

        assert!(num_levels >= 1);
        TreeBuildVisitor {
            bots,
            current_height: num_levels - 1,
            axis: default_axis().to_dyn(),
        }
    }
    #[must_use]
    pub fn get_height(&self) -> usize {
        self.current_height
    }
    #[must_use]
    pub fn build_and_next(self) -> NodeBuildResult<'a, T> {
        //leaf case
        if self.current_height == 0 {
            let node = NodeFinisher {
                middle_left_len: None,
                mid: self.bots,
                div: None,
                axis: self.axis,
                min_elem: 0,
                num_elem: 0,
            };

            NodeBuildResult { node, rest: None }
        } else {
            fn construct_non_leaf<T: Aabb>(
                div_axis: impl Axis,
                bots: &mut [T],
            ) -> (NodeFinisher<T>, &mut [T], &mut [T]) {
                if bots.is_empty() {
                    return (
                        NodeFinisher {
                            middle_left_len: None,
                            mid: bots,
                            div: None,
                            axis: div_axis.to_dyn(),
                            min_elem: 0,
                            num_elem: 0,
                        },
                        &mut [],
                        &mut [],
                    );
                }

                let med_index = bots.len() / 2;

                let (ll, med, rr) = bots.select_nth_unstable_by(med_index, move |a, b| {
                    crate::queries::cmp_aabb(div_axis, a, b)
                });

                let med_val = med.range(div_axis).start;

                let (ml, ll) = {
                    let mut m = 0;
                    for a in 0..ll.len() {
                        if ll[a].range(div_axis).end >= med_val {
                            //keep
                            ll.swap(a, m);
                            m += 1;
                        }
                    }
                    ll.split_at_mut(m)
                };

                let (mr, rr) = {
                    let mut m = 0;
                    for a in 0..rr.len() {
                        if rr[a].range(div_axis).start <= med_val {
                            //keep
                            rr.swap(a, m);
                            m += 1;
                        }
                    }
                    rr.split_at_mut(m)
                };

                let ml_len = ml.len();
                let ll_len = ll.len();
                let rr_len = rr.len();

                let mr_len = mr.len();

                {
                    let (_, rest) = bots.split_at_mut(ml_len);
                    let (ll, rest) = rest.split_at_mut(ll_len);
                    let (mr2, _) = rest.split_at_mut(1 + mr_len);

                    let (a, b) = if mr2.len() < ll.len() {
                        let (a, _) = ll.split_at_mut(mr2.len());
                        (a, mr2)
                    } else {
                        let (_, a) = mr2.split_at_mut(mr2.len() - ll.len());
                        (a, ll)
                    };
                    a.swap_with_slice(b);
                }

                let left_len = ll_len;
                let right_len = rr_len;
                let mid_len = ml_len + 1 + mr_len;

                let middle_left_len = Some(ml_len + 1);

                let (mid, rest) = bots.split_at_mut(mid_len);
                let (left, right) = rest.split_at_mut(left_len);

                (
                    NodeFinisher {
                        middle_left_len,
                        mid,
                        div: Some(med_val),
                        axis: div_axis.to_dyn(),
                        min_elem: left_len.min(right_len),
                        num_elem: left_len + right_len,
                    },
                    left,
                    right,
                )
            }

            let (finish_node, left, right) = match self.axis {
                AxisDyn::X => construct_non_leaf(axgeom::XAXIS, self.bots),
                AxisDyn::Y => construct_non_leaf(axgeom::YAXIS, self.bots),
            };

            NodeBuildResult {
                node: finish_node,
                rest: Some([
                    TreeBuildVisitor {
                        bots: left,
                        current_height: self.current_height.saturating_sub(1),
                        axis: self.axis.next(),
                    },
                    TreeBuildVisitor {
                        bots: right,
                        current_height: self.current_height.saturating_sub(1),
                        axis: self.axis.next(),
                    },
                ]),
            }
        }
    }

    pub fn recurse_seq<S: Sorter<T>>(self, sorter: &mut S, buffer: &mut Vec<Node<'a, T, T::Num>>) {
        let NodeBuildResult { node, rest } = self.build_and_next();
        buffer.push(node.finish(sorter));
        if let Some([left, right]) = rest {
            left.recurse_seq(sorter, buffer);
            right.recurse_seq(sorter, buffer);
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct DefaultSorter;

impl<T: Aabb> Sorter<T> for DefaultSorter {
    fn sort(&self, axis: impl Axis, bots: &mut [T]) {
        crate::build::sweeper_update(axis, bots);
    }
}
