use super::*;

pub enum GravEnum<'a, T: Aabb, M> {
    Mass(&'a mut M),
    Bot(PMut<'a, [T]>),
}

pub trait NNN {
    type T: Aabb<Num=Self::N>;
    type N: Num;
    type Mass: Default + Copy + core::fmt::Debug;

    //return the position of the center of mass
    fn compute_center_of_mass(&mut self, a: &[Self::T]) -> Self::Mass;


    fn is_close(&mut self,a:&Self::Mass,line:Self::N,a:impl Axis)->bool;
    fn is_close_half(&mut self,a:&Self::Mass,line:Self::N,a:impl Axis)->bool;


    
    fn gravitate(&mut self, a: GravEnum<Self::T, Self::Mass>, b: GravEnum<Self::T, Self::Mass>);

    fn gravitate_self(&mut self, a: PMut<[Self::T]>);

    fn apply_a_mass(&mut self, mass: Self::Mass, i: PMut<[Self::T]>);

    fn combine_two_masses(&mut self, a: &Self::Mass, b: &Self::Mass) -> Self::Mass;
}

use compt::dfs_order::CompleteTreeContainer;
use compt::dfs_order::PreOrder;
use compt::dfs_order::VistrMut;

struct NodeWrapper<'a, T: Aabb, M> {
    node: Node<'a, T>,
    mass: M,
}



/*
struct NbodyHandler<'a,N:NNN>{
    n:&'a mut N
}

impl NbodyHandler{

}
*/

fn build_masses2<N: NNN>(
    vistr: VistrMut<NodeWrapper<N::T, N::Mass>, PreOrder>,
    no: &mut N,
) -> N::Mass {
    let (nn, rest) = vistr.next();
    let mass = no.compute_center_of_mass(&nn.node.range);
    let mass = if let Some([left, right]) = rest {
        let a = build_masses2(left, no);
        let b = build_masses2(right, no);
        let m = no.combine_two_masses(&a, &b);
        no.combine_two_masses(&m, &mass)
    } else {
        mass
    };
    nn.mass = mass;
    mass
}

fn collect_masses<'a, 'b, N: NNN>(
    root_div:N::N,
    root_axis:impl Axis,
    root: &N::Mass,
    vistr: VistrMut<'b, NodeWrapper<'a, N::T, N::Mass>, PreOrder>,
    no: &mut N,
    mut func1: &mut impl FnMut(&'b mut NodeWrapper<'a, N::T, N::Mass>, &mut N),
    mut func2: &mut impl FnMut(&'b mut PMut<'a, [N::T]>, &mut N),
) {
    let (nn, rest) = vistr.next();

    
    if !no.is_close_half(&nn.mass,root_div,root_axis){
        func1(nn,no);
        return;
    }
    /*
    if !no.are_close(root, &nn.mass) {
        func1(nn, no);
        return;
    }
    */

    func2(&mut nn.node.range, no);
    
    if let Some([mut left, mut right]) = rest {
        collect_masses(root_div,root_axis,root, left, no, func1, func2);
        collect_masses(root_div,root_axis,root, right, no, func1, func2);
    }
}

fn pre_recc<N: NNN>(
    root_div:N::N,
    root_axis:impl Axis,
    root: &mut NodeWrapper<N::T, N::Mass>,
    vistr: VistrMut<NodeWrapper<N::T, N::Mass>, PreOrder>,
    no: &mut N,
) {
    let (nn, rest) = vistr.next();
    
    if !no.is_close(&nn.mass,root_div,root_axis){
        no.gravitate(
            GravEnum::Bot(root.node.range.borrow_mut()),
            GravEnum::Mass(&mut nn.mass),
        );
        return;
    }
    
    /*
    if !no.are_close(&root.mass, &nn.mass) {
        no.gravitate(
            GravEnum::Bot(root.node.range.borrow_mut()),
            GravEnum::Mass(&mut nn.mass),
        );
        return;
    }
    */
    

    no.gravitate(
        GravEnum::Bot(root.node.range.borrow_mut()),
        GravEnum::Bot(nn.node.range.borrow_mut()),
    );

    if let Some([mut left, mut right]) = rest {
        pre_recc(root_div,root_axis,root, left, no);
        pre_recc(root_div,root_axis,root, right, no);
    }
}

fn recc<N: NNN>(
    axis: impl Axis,
    vistr: VistrMut<NodeWrapper<N::T, N::Mass>, PreOrder>,
    no: &mut N,
) {
    let (nn, rest) = vistr.next();

    no.gravitate_self(nn.node.range.borrow_mut());

    if let Some([mut left, mut right]) = rest {
        
        if let Some(div) = nn.node.div {
            pre_recc(div,axis,nn, left.borrow_mut(), no);
            pre_recc(div,axis,nn, right.borrow_mut(), no);

            
            let mut finished_masses = Vec::new();
            let mut finished_bots = Vec::new();

            collect_masses(
                div,
                axis,
                &nn.mass,
                left.borrow_mut(),
                no,
                &mut |a, _| finished_masses.push(a),
                &mut |a, _| finished_bots.push(a),
            );

            let mut finished_masses2 = Vec::new();
            let mut finished_bots2 = Vec::new();

            collect_masses(
                div,
                axis,
                &nn.mass,
                right.borrow_mut(),
                no,
                &mut |a, _| finished_masses2.push(a),
                &mut |a, _| finished_bots2.push(a),
            );

            //We have collected masses on both sides.
            //now gravitate all the ones on the left side with all the ones on the right side.

            for a in finished_masses.into_iter() {
                for b in finished_masses2.iter_mut() {
                    no.gravitate(GravEnum::Mass(&mut a.mass), GravEnum::Mass(&mut b.mass));
                }

                for b in finished_bots2.iter_mut() {
                    no.gravitate(GravEnum::Mass(&mut a.mass), GravEnum::Bot(b.borrow_mut()));
                }
            }
            for a in finished_bots.into_iter() {
                for b in finished_masses2.iter_mut() {
                    no.gravitate(GravEnum::Bot(a.borrow_mut()), GravEnum::Mass(&mut b.mass));
                }
                for b in finished_bots2.iter_mut() {
                    no.gravitate(GravEnum::Bot(a.borrow_mut()), GravEnum::Bot(b.borrow_mut()));
                }
            }

            //parallelize this
            recc(axis.next(), left, no);
            recc(axis.next(), right, no);
        }
    }
}

fn get_bots_from_vistr<'a, T: Aabb, N>(
    mut vistr: VistrMut<'a, NodeWrapper<T, N>, PreOrder>,
) -> PMut<'a, [T]> {
    let mut new_slice = None;

    vistr.dfs_preorder(|a| {
        if let Some(s) = new_slice.take() {
            new_slice = Some(crate::pmut::combine_slice(s, a.node.range.borrow_mut()));
        } else {
            new_slice = Some(a.node.range.borrow_mut());
        }
    });
    new_slice.unwrap()
}
fn apply_tree<N: NNN>(mut vistr: VistrMut<NodeWrapper<N::T, N::Mass>, PreOrder>, no: &mut N) {
    {
        let mass = vistr.borrow_mut().next().0.mass;

        let new_slice = get_bots_from_vistr(vistr.borrow_mut());

        no.apply_a_mass(mass, new_slice);
    }

    let (nn, rest) = vistr.next();

    if let Some([mut left, mut right]) = rest {
        apply_tree(left, no);
        apply_tree(right, no);
    }
}

//TODO work on this!!!
pub fn nbody_mut<'a, N: NNN>(tree: crate::Tree<'a, N::T>, mut no: &mut N) -> crate::Tree<'a, N::T> {
    let t: CompleteTreeContainer<Node<N::T>, PreOrder> = tree.inner;

    let k = t
        .into_nodes()
        .into_vec()
        .into_iter()
        .map(|node| NodeWrapper {
            node,
            mass: Default::default(),
        })
        .collect();

    let mut newtree = CompleteTreeContainer::from_preorder(k).unwrap();

    //calculate node masses of each node.
    build_masses2(newtree.vistr_mut(), no);

    recc(default_axis(), newtree.vistr_mut(), no);

    apply_tree(newtree.vistr_mut(), no);

    let nt: Vec<_> = newtree
        .into_nodes()
        .into_vec()
        .into_iter()
        .map(|mut node| node.node)
        .collect();

    let mut inner = CompleteTreeContainer::from_preorder(nt).unwrap();

    crate::Tree { inner }
}
