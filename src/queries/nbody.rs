//!
//! Experimental nbody approximate solver
//!
//! The user can choose the distance at which to fallback on approximate solutions.
//! The algorithm works similar to a Barnes–Hut simulation, but uses a kdtree instead of a quad tree.
//!
//! The user defines some geometric functions and their ideal accuracy.
//!
use super::*;

type NodeWrapperVistr<'a, 'b, T, M> = VistrMut<'a, NodeWrapper<'b, T, M>, PreOrder>;

///Helper enum indicating whether or not to gravitate a node as a whole, or as its individual parts.
pub enum GravEnum<'a, T: Aabb, M> {
    Mass(&'a mut M),
    Bot(AabbPin<&'a mut [T]>),
}

///User defined functions for nbody
pub trait Nbody {
    type T: Aabb<Num = Self::N>;
    type N: Num;
    type Mass: Default + Copy + core::fmt::Debug;

    //return the position of the center of mass
    fn compute_center_of_mass(&mut self, a: &[Self::T]) -> Self::Mass;

    fn is_close(&self, a: &Self::Mass, line: Self::N, a: impl Axis) -> bool;

    fn is_close_half(&self, a: &Self::Mass, line: Self::N, a: impl Axis) -> bool;

    fn gravitate(&mut self, a: GravEnum<Self::T, Self::Mass>, b: GravEnum<Self::T, Self::Mass>);

    fn gravitate_self(&mut self, a: AabbPin<&mut [Self::T]>);

    fn apply_a_mass<'a>(
        &'a mut self,
        mass: Self::Mass,
        i: impl Iterator<Item = AabbPin<&'a mut Self::T>>,
        len: usize,
    );

    fn combine_two_masses(&mut self, a: &Self::Mass, b: &Self::Mass) -> Self::Mass;
}

use compt::dfs_order::PreOrder;
use compt::dfs_order::VistrMut;

struct NodeWrapper<'a, T: Aabb, M> {
    node: Node<'a, T>,
    mass: M,
}

fn build_masses2<N: Nbody>(vistr: NodeWrapperVistr<N::T, N::Mass>, no: &mut N) -> N::Mass {
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
    vistr: NodeWrapperVistr<'b, 'a, N::T, N::Mass>,
    no: &mut N,
    func1: &mut impl FnMut(&'b mut NodeWrapper<'a, N::T, N::Mass>, &mut N),
    func2: &mut impl FnMut(&'b mut AabbPin<&'a mut [N::T]>, &mut N),
) {
    let (nn, rest) = vistr.next();

    if !no.is_close_half(&nn.mass, root_div, root_axis) {
        func1(nn, no);
        return;
    }

    func2(&mut nn.node.range, no);

    if let Some([left, right]) = rest {
        collect_masses(root_div, root_axis, left, no, func1, func2);
        collect_masses(root_div, root_axis, right, no, func1, func2);
    }
}

fn pre_recc<N: Nbody>(
    root_div: N::N,
    root_axis: impl Axis,
    root: &mut NodeWrapper<N::T, N::Mass>,
    vistr: NodeWrapperVistr<N::T, N::Mass>,
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
    vistr: NodeWrapperVistr<'a, 'b, N::T, N::Mass>,
    no: &mut N,
) -> Option<[NodeWrapperVistr<'a, 'b, N::T, N::Mass>; 2]> {
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

fn apply_tree<N: Nbody>(mut vistr: NodeWrapperVistr<N::T, N::Mass>, no: &mut N) {
    {
        let mass = vistr.borrow_mut().next().0.mass;

        let len = vistr
            .borrow_mut()
            .dfs_preorder_iter()
            .map(|x| x.node.range.borrow_mut().len())
            .sum();

        let it = vistr
            .borrow_mut()
            .dfs_preorder_iter()
            .flat_map(|x| x.node.range.borrow_mut().iter_mut());

        no.apply_a_mass(mass, it, len);
    }

    let (_, rest) = vistr.next();

    if let Some([left, right]) = rest {
        apply_tree(left, no);
        apply_tree(right, no);
    }
}

impl<'a, T: Aabb> crate::Tree2<'a, T> {
    fn handle_nbody<N: Nbody<T = T>>(self, no: &mut N) -> Self {
        ///Perform nbody
        ///The tree is taken by value so that its nodes can be expended to include more data.
        pub fn nbody_mut<'a, N: Nbody>(
            tree: Vec<Node<'a, N::T>>,
            no: &mut N,
        ) -> Vec<Node<'a, N::T>> {
            let mut newnodes: Vec<_> = tree
                .into_iter()
                .map(|x| NodeWrapper {
                    node: x,
                    mass: Default::default(),
                })
                .collect();

            let tree = compt::dfs_order::CompleteTreeMut::from_preorder_mut(&mut newnodes).unwrap();
            let mut vistr = tree.vistr_mut();

            //calculate node masses of each node.
            build_masses2(vistr.borrow_mut(), no);

            recc(default_axis(), vistr.borrow_mut(), no);

            apply_tree(vistr, no);

            newnodes.into_iter().map(|x| x.node).collect()
        }

        Tree2 {
            nodes: nbody_mut(self.nodes, no),
            total_num_elem: self.total_num_elem,
        }
    }
}

impl<'a, T: Aabb> Naive<'a, T> {
    pub fn handle_nbody<N: Nbody<T = T>>(mut self, no: &mut N) -> Self {
        ///Naive version simply visits every pair.
        pub fn naive_nbody_mut<T: Aabb>(
            bots: AabbPin<&mut [T]>,
            func: impl FnMut(AabbPin<&mut T>, AabbPin<&mut T>),
        ) {
            queries::for_every_pair(bots, func);
        }

        naive_nbody_mut(self.inner.borrow_mut(), |a, b| {
            no.gravitate(GravEnum::Bot(a.into_slice()), GravEnum::Bot(b.into_slice()));
        });
        self
    }
}
