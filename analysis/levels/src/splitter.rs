use support::broccoli;

///A trait that gives the user callbacks at events in a recursive algorithm on the tree.
///The main motivation behind this trait was to track the time spent taken at each level of the tree
///during construction.
pub trait Splitter: Sized {
    ///Called to split this into two to be passed to the children.
    fn div(&mut self) -> Self;

    ///Called to add the results of the recursive calls on the children.
    fn add(&mut self, b: Self);
}

pub struct EmptySplitter;

impl Splitter for EmptySplitter {
    fn div(&mut self) -> Self {
        EmptySplitter
    }
    fn add(&mut self, _: Self) {}
}

pub mod build {
    use super::*;
    use broccoli::aabb::*;
    use broccoli::tree::{
        build::{NodeBuildResult, TreeBuildVisitor},
        node::Node,
        Sorter,
    };

    pub fn recurse_seq_splitter<'a, T: Aabb + ManySwap, S: Sorter<T>, P: Splitter>(
        vistr: TreeBuildVisitor<'a, T>,
        splitter: &mut P,
        sorter: &mut S,
        buffer: &mut Vec<Node<'a, T, T::Num>>,
    ) {
        let NodeBuildResult { node, rest } = vistr.build_and_next();
        buffer.push(node.finish(sorter));
        if let Some([left, right]) = rest {
            let mut a = splitter.div();

            recurse_seq_splitter(left, splitter, sorter, buffer);

            recurse_seq_splitter(right, &mut a, sorter, buffer);
            splitter.add(a);
        }
    }
}

pub mod query {
    use super::*;
    pub mod colfind {
        use super::*;
        use broccoli::{
            aabb::*,
            queries::colfind::build::{CollVis, NodeHandler},
        };

        pub fn recurse_seq_splitter<T: Aabb, P: Splitter, N: NodeHandler<T>>(
            vistr: CollVis<T>,
            splitter: &mut P,
            func: &mut N,
        ) {
            let (n, rest) = vistr.collide_and_next(func);

            if let Some([left, right]) = rest {
                let mut s2 = splitter.div();
                n.finish(func);
                recurse_seq_splitter(left, splitter, func);
                recurse_seq_splitter(right, &mut s2, func);
                splitter.add(s2);
            } else {
                n.finish(func);
            }
        }
    }
}
