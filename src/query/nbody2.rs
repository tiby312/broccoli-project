use super::*;


/*
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
*/

pub enum GravEnum<'a,T:Aabb,M>{
    Mass(&'a mut M),
    Bot(PMut<'a,[T]>)
}

pub trait NNN{
    type T:Aabb;
    type N:Num;
    type Mass:Default+Copy;

    //return the position of the center of mass
    fn compute_center_of_mass(&mut self,a:&[Self::T])->Self::Mass;

    fn are_close(&mut self,a:&Self::Mass,b:&Self::Mass)->bool;


    fn gravitate(&mut self,a:GravEnum<Self::T,Self::Mass>,b:GravEnum<Self::T,Self::Mass>);

    fn gravitate_self(&mut self,a:PMut<[Self::T]>);

    fn apply_a_mass(&mut self,mass:Self::Mass,b:PMut<[Self::T]>);

    fn combine_two_masses(&mut self,a:&Self::Mass,b:&Self::Mass)->Self::Mass;
}

use compt::dfs_order::CompleteTreeContainer;
use compt::dfs_order::PreOrder;
use compt::dfs_order::VistrMut;


struct NodeWrapper<'a,T:Aabb,M>{
    node:Node<'a,T>,
    mass:M
}


fn build_masses<N:NNN>(vistr:VistrMut<NodeWrapper<N::T,N::Mass>,PreOrder>,no:&mut N)->N::Mass{
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

fn pre_recc<N:NNN>(root:&mut NodeWrapper<N::T,N::Mass>,vistr:VistrMut<NodeWrapper<N::T,N::Mass>,PreOrder>,no:&mut N){
    let (nn,rest)=vistr.next();

    if !no.are_close(&root.mass,&nn.mass){
        no.gravitate(
            GravEnum::Mass(&mut root.mass),
            GravEnum::Mass(&mut nn.mass)
        );
        return
    }

    no.gravitate(
        GravEnum::Mass(&mut root.mass),
        GravEnum::Bot(nn.node.range.borrow_mut())
    );

    if let Some([mut left,mut right])=rest{
        pre_recc(root,left,no);
        pre_recc(root,right,no);
    }
}

fn collect_masses<'a,'b,N:NNN>(
    root:&N::Mass,
    vistr:VistrMut<'b,NodeWrapper<'a,N::T,N::Mass>,PreOrder>,
    no:&mut N,
    finished_mass:&mut Vec<&'b mut NodeWrapper<'a,N::T,N::Mass>>,
    finished_bots:&mut Vec<&'b mut PMut<'a,[N::T]>>){

    let (nn,rest)=vistr.next();

    if !no.are_close(root,&nn.mass){
        finished_mass.push(nn);
        return;
    }

    finished_bots.push(&mut nn.node.range);

    if let Some([mut left,mut right])=rest{
        collect_masses(root,left,no,finished_mass,finished_bots);
        collect_masses(root,right,no,finished_mass,finished_bots);
    }
}
fn recc<N:NNN>(vistr:VistrMut<NodeWrapper<N::T,N::Mass>,PreOrder>,no:&mut N){

    let (nn,rest)=vistr.next();

    no.gravitate_self(nn.node.range.borrow_mut());
    
    if let Some([mut left,mut right])=rest{
        pre_recc(nn,left.borrow_mut(),no);
        pre_recc(nn,right.borrow_mut(),no);

        if let Some(div)=nn.node.div{

            let mut finished_masses=Vec::new();
            let mut finished_bots=Vec::new();
            
            collect_masses(&nn.mass,left.borrow_mut(),no,&mut finished_masses,&mut finished_bots);

            let mut finished_masses2=Vec::new();
            let mut finished_bots2=Vec::new();
            
            collect_masses(&nn.mass,right.borrow_mut(),no,&mut finished_masses2,&mut finished_bots2);

            //We have collected masses on both sides.
            //now gravitate all the ones on the left side with all the ones on the right side.

            for a in finished_masses.into_iter(){
                for b in finished_masses2.iter_mut(){
                    no.gravitate(GravEnum::Mass(&mut a.mass),GravEnum::Mass(&mut b.mass));
                }
                
                for b in finished_bots2.iter_mut(){
                    no.gravitate(GravEnum::Mass(&mut a.mass),GravEnum::Bot(b.borrow_mut()));
                }
                
            }
            for a in finished_bots.into_iter(){
                for b in finished_masses2.iter_mut(){
                    no.gravitate(GravEnum::Bot(a.borrow_mut()),GravEnum::Mass(&mut b.mass));
                  
                }
                for b in finished_bots2.iter_mut(){
                    no.gravitate(GravEnum::Bot(a.borrow_mut()),GravEnum::Bot(b.borrow_mut()));
              
                }
            }
            recc(left,no);
            recc(right,no);
        }
    }   
}


//TODO work on this!!!
fn nbody2<N:NNN>(tree:crate::Tree<N::T>,mut no:N)->crate::Tree<N::T>
{
   
    let t:CompleteTreeContainer<Node<N::T>, PreOrder>=tree.inner;


    let k=t.into_nodes().into_vec().into_iter().map(|node|NodeWrapper{node,mass:Default::default()}).collect();

    let mut newtree=CompleteTreeContainer::from_preorder(k).unwrap();

    //calculate node masses of each node.
    build_masses(newtree.vistr_mut(),&mut no);

    recc(newtree.vistr_mut(),&mut no);
    
    let nt:Vec<_>=newtree.into_nodes().into_vec().into_iter().map(|mut node|{
        no.apply_a_mass(node.mass,node.node.range.borrow_mut());
        node.node
    }).collect();

    let mut inner=CompleteTreeContainer::from_preorder(nt).unwrap();

    crate::Tree{
        inner
    }
}
    