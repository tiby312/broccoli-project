use super::*;


#[derive(Copy,Clone)]
struct Mass<N:Num>{
    pos:Vec2<N>,
    mass:N
}

use core::default::Default;
impl<N:Num+Default> Mass<N>{
    fn new()->Mass<N>{
        Mass{
            pos:vec2(Default::default(),Default::default()),
            mass:Default::default()
        }
    }
}

trait NNN{
    type T:Aabb;
    type N:Num;

    //return the position of the center of mass
    fn compute_center_of_mass(&mut self,a:&[Self::T])->Mass<Self::N>;

    fn are_close(&mut self,a:&Mass<Self::N>,b:&Mass<Self::N>)->bool;

    fn gravitate(&mut self,a:(Option<&mut Mass<Self::N>>,PMut<[Self::T]>),b:(Option<&mut Mass<Self::N>>,PMut<[Self::T]>));


    fn combine_two_masses(&mut self,a:&Mass<Self::N>,b:&Mass<Self::N>)->Mass<Self::N>;
}

use compt::dfs_order::CompleteTreeContainer;
use compt::dfs_order::PreOrder;
use compt::dfs_order::VistrMut;


struct NodeWrapper<'a,T:Aabb>{
    node:Node<'a,T>,
    mass:Mass<T::Num>
}


fn build_masses<T:Aabb>(vistr:VistrMut<NodeWrapper<T>,PreOrder>,no:&mut impl NNN<T=T,N=T::Num>)->Mass<T::Num>{
    let (nn,rest)=vistr.next();
    let mass=no.compute_center_of_mass(&nn.node.range);
    let mass=if let Some([left,right])=rest{
        let a=build_masses(left,no);
        let b=build_masses(right,no);
        let m=no.combine_two_masses(&a,&b);
        no.combine_two_masses(&m,&mass)
    }else{
        mass
    };
    nn.mass=mass;
    mass
}

fn pre_recc<T:Aabb>(root:&mut NodeWrapper<T>,vistr:VistrMut<NodeWrapper<T>,PreOrder>,no:&mut impl NNN<T=T,N=T::Num>){
    let (nn,rest)=vistr.next();

    if !no.are_close(&root.mass,&nn.mass){
        no.gravitate(
            (Some(&mut root.mass),root.node.range.borrow_mut()),
            (Some(&mut nn.mass), nn.node.range.borrow_mut())
        );
        return
    }

    no.gravitate(
        (Some(&mut root.mass),root.node.range.borrow_mut()),
        (None,nn.node.range.borrow_mut())
    );

    if let Some([mut left,mut right])=rest{
        pre_recc(root,left,no);
        pre_recc(root,right,no);
    }
}
fn recc<T:Aabb>(vistr:VistrMut<NodeWrapper<T>,PreOrder>,no:&mut impl NNN<T=T,N=T::Num>){

    let (nn,rest)=vistr.next();

    if let Some([mut left,mut right])=rest{
        pre_recc(nn,left.borrow_mut(),no);
        pre_recc(nn,right.borrow_mut(),no);
    }else{
        unimplemented!()
    }   

}


//TODO work on this!!!
fn nbody2<T:Aabb,NO>(tree:crate::Tree<T>,mut no:NO)
    where NO:NNN<T=T,N=T::Num>, T::Num:Default
{
   
    let t:CompleteTreeContainer<Node<T>, PreOrder>=tree.inner;


    let k=t.into_nodes().into_vec().into_iter().map(|node|NodeWrapper{node,mass:Mass::new()}).collect();

    let mut newtree=CompleteTreeContainer::from_preorder(k).unwrap();

    //calculate node masses of each node.
    build_masses(newtree.vistr_mut(),&mut no);

    



}
    