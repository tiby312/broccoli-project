//!
//! Experimental nbody approximate solver
//!
//! The user can choose the distance at which to fallback on approximate solutions.
//! The algorithm works similar to a Barnes–Hut simulation, but uses a kdtree instead of a quad tree.
//!
//! The user defines some geometric functions and their ideal accuracy.
//!
use super::*;

///Helper enum indicating whether or not to gravitate a node as a whole, or as its individual parts.
pub enum GravEnum<'a, T: Aabb, M> {
    Mass(&'a mut M),
    Bot(PMut<'a, [T]>),
}

///User defined functions for nbody
pub trait Nbody {
    type T: Aabb<Num = Self::N>;
    type N: Num;
    type Mass: Default + Copy + core::fmt::Debug;

    //return the position of the center of mass
    fn compute_center_of_mass(&mut self, a: &[Self::T]) -> Self::Mass;

    fn is_close(&mut self, a: &Self::Mass, line: Self::N, a: impl Axis) -> bool;

    fn is_close_half(&mut self, a: &Self::Mass, line: Self::N, a: impl Axis) -> bool;

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

///Naive version simply visits every pair.
pub fn naive_mut<T: Aabb>(bots: PMut<[T]>, func: impl FnMut(PMut<T>, PMut<T>)) {
    tools::for_every_pair(bots, func);
}

fn build_masses2<N: Nbody>(
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

fn collect_masses<'a, 'b, N: Nbody>(
    root_div: N::N,
    root_axis: impl Axis,
    root: &N::Mass,
    vistr: VistrMut<'b, NodeWrapper<'a, N::T, N::Mass>, PreOrder>,
    no: &mut N,
    func1: &mut impl FnMut(&'b mut NodeWrapper<'a, N::T, N::Mass>, &mut N),
    func2: &mut impl FnMut(&'b mut PMut<'a, [N::T]>, &mut N),
) {
    let (nn, rest) = vistr.next();

    if !no.is_close_half(&nn.mass, root_div, root_axis) {
        func1(nn, no);
        return;
    }

    func2(&mut nn.node.range, no);

    if let Some([left, right]) = rest {
        collect_masses(root_div, root_axis, root, left, no, func1, func2);
        collect_masses(root_div, root_axis, root, right, no, func1, func2);
    }
}

fn pre_recc<N: Nbody>(
    root_div: N::N,
    root_axis: impl Axis,
    root: &mut NodeWrapper<N::T, N::Mass>,
    vistr: VistrMut<NodeWrapper<N::T, N::Mass>, PreOrder>,
    no: &mut N,
) {
    let (nn, rest) = vistr.next();

    if !no.is_close(&nn.mass, root_div, root_axis) {
        no.gravitate(
            GravEnum::Bot(root.node.range.borrow_mut()),
            GravEnum::Mass(&mut nn.mass),
        );
        return;
    }

    no.gravitate(
        GravEnum::Bot(root.node.range.borrow_mut()),
        GravEnum::Bot(nn.node.range.borrow_mut()),
    );

    if let Some([left, right]) = rest {
        pre_recc(root_div, root_axis, root, left, no);
        pre_recc(root_div, root_axis, root, right, no);
    }
}

fn recc_common<'a, 'b, N: Nbody>(
    axis: impl Axis,
    vistr: VistrMut<'a, NodeWrapper<'b, N::T, N::Mass>, PreOrder>,
    no: &mut N,
) -> Option<[VistrMut<'a, NodeWrapper<'b, N::T, N::Mass>, PreOrder>; 2]> {
    let (nn, rest) = vistr.next();

    no.gravitate_self(nn.node.range.borrow_mut());

    if let Some([mut left, mut right]) = rest {
        if let Some(div) = nn.node.div {
            pre_recc(div, axis, nn, left.borrow_mut(), no);
            pre_recc(div, axis, nn, right.borrow_mut(), no);

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
            Some([left, right])
        } else {
            None
        }
    } else {
        None
    }
}

fn recc_par<N: Nbody, JJ: par::Joiner>(
    axis: impl Axis,
    par: JJ,
    vistr: VistrMut<NodeWrapper<N::T, N::Mass>, PreOrder>,
    no: &mut N,
) where
    N::T: Send,
    N::N: Send,
    N::Mass: Send,
    N: Splitter + Send + Sync,
{
    let keep_going = recc_common(axis, vistr, no);

    if let Some([left, right]) = keep_going {
        match par.next() {
            par::ParResult::Parallel([dleft, dright]) => {
                let (mut no1, mut no2) = no.div();
                rayon::join(
                    || recc_par(axis.next(), dleft, left, &mut no1),
                    || recc_par(axis.next(), dright, right, &mut no2),
                );
                no.add(no1, no2);
            }
            par::ParResult::Sequential(_) => {
                recc(axis.next(), left, no);
                recc(axis.next(), right, no);
            }
        }
    }
}

fn recc<N: Nbody>(
    axis: impl Axis,
    vistr: VistrMut<NodeWrapper<N::T, N::Mass>, PreOrder>,
    no: &mut N,
) {
    let keep_going = recc_common(axis, vistr, no);

    if let Some([left, right]) = keep_going {
        recc(axis.next(), left, no);
        recc(axis.next(), right, no);
    }
}

fn get_bots_from_vistr<'a, T: Aabb, N>(
    vistr: VistrMut<'a, NodeWrapper<T, N>, PreOrder>,
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
fn apply_tree<N: Nbody>(mut vistr: VistrMut<NodeWrapper<N::T, N::Mass>, PreOrder>, no: &mut N) {
    {
        let mass = vistr.borrow_mut().next().0.mass;

        let new_slice = get_bots_from_vistr(vistr.borrow_mut());

        no.apply_a_mass(mass, new_slice);
    }

    let (_, rest) = vistr.next();

    if let Some([left, right]) = rest {
        apply_tree(left, no);
        apply_tree(right, no);
    }
}

type TreeInner<T> = CompleteTreeContainer<T, PreOrder>;

fn convert_tree_into_wrapper<T: Aabb, M: Default>(
    tree: TreeInner<Node<T>>,
) -> TreeInner<NodeWrapper<T, M>> {
    let k = tree
        .into_nodes()
        .into_vec()
        .into_iter()
        .map(|node| NodeWrapper {
            node,
            mass: Default::default(),
        })
        .collect();

    CompleteTreeContainer::from_preorder(k).unwrap()
}
fn convert_wrapper_into_tree<T: Aabb, M: Default>(
    tree: TreeInner<NodeWrapper<T, M>>,
) -> TreeInner<Node<T>> {
    let nt: Vec<_> = tree
        .into_nodes()
        .into_vec()
        .into_iter()
        .map(|node| node.node)
        .collect();

    CompleteTreeContainer::from_preorder(nt).unwrap()
}

///Perform nbody
///The tree is taken by value so that its nodes can be expended to include more data.
pub fn nbody_mut_par<'a, N: Nbody>(tree: crate::Tree<'a, N::T>, no: &mut N) -> crate::Tree<'a, N::T>
where
    N: Send + Sync + Splitter,
    N::T: Send + Sync,
    <N::T as Aabb>::Num: Send + Sync,
    N::Mass: Send + Sync,
{
    let mut newtree = convert_tree_into_wrapper(tree.inner);

    //calculate node masses of each node.
    build_masses2(newtree.vistr_mut(), no);


    let par=par::ParallelBuilder::new().build_for_tree_of_height(newtree.get_height());

    recc_par(default_axis(), par, newtree.vistr_mut(), no);

    apply_tree(newtree.vistr_mut(), no);

    crate::Tree {
        inner: convert_wrapper_into_tree(newtree),
    }
}

///Perform nbody
///The tree is taken by value so that its nodes can be expended to include more data.
pub fn nbody_mut<'a, N: Nbody>(tree: crate::Tree<'a, N::T>, no: &mut N) -> crate::Tree<'a, N::T> {
    let mut newtree = convert_tree_into_wrapper(tree.inner);

    //calculate node masses of each node.
    build_masses2(newtree.vistr_mut(), no);

    recc(default_axis(), newtree.vistr_mut(), no);

    apply_tree(newtree.vistr_mut(), no);

    crate::Tree {
        inner: convert_wrapper_into_tree(newtree),
    }
}
