use broccoli::{
    tree::{
        build::{DefaultSorter, NodeBuildResult, TreeBuildVisitor},
        node::{Aabb, ManySwap, Node},
        splitter::{EmptySplitter, Splitter},
        Sorter,
    },
    Tree,
};

use broccoli::tree::num_level;

pub trait RayonBuildPar<'a, T: Aabb> {
    fn par_new_ext<P: Splitter>(
        bots: &'a mut [T],
        num_level: usize,
        splitter: P,
        num_seq_fallback: usize,
    ) -> (Self, P);
    fn par_new(bots: &'a mut [T]) -> Self;
}

impl<'a, T: Aabb> RayonBuildPar<'a, T> for Tree<'a, T> {
    fn par_new_ext<P: Splitter>(
        bots: &'a mut [T],
        num_level: usize,
        mut splitter: P,
        num_seq_fallback: usize,
    ) -> (Self, P) {
        let mut buffer = Vec::with_capacity(num_level::num_nodes(num_level));
        recurse_par(
            num_seq_fallback,
            &mut splitter,
            &mut DefaultSorter,
            &mut buffer,
            TreeBuildVisitor::new(num_level, bots),
        );
        (buffer, splitter)
    }
    fn par_new(bots: &'a mut [T]) -> Self {
        let num_level = num_level::default(bots.len());
        Self::par_new_ext(bots, num_level, EmptySplitter, 512).0
    }
}

// we want to pass small chunks so that if a slow core
// gets a task, they don't hold everybody else up.

// at the same time, we don't want there to be only
// a few chunks. i.e. only 3 cores available but 4 chunks.

// so lets only result to chunking IF
// the problem size is big enough such that there
// are many chunks.

fn recurse_par<'a, T: Aabb + ManySwap, S: Sorter<T>, P: Splitter>(
    num_seq_fallback: usize,
    splitter: &mut P,
    sorter: &mut S,
    buffer: &mut Vec<Node<'a, T>>,
    vistr: TreeBuildVisitor<'a, T>,
) where
    S: Send,
    T: Send,
    T::Num: Send,
    P: Send,
{
    let NodeBuildResult { node, rest } = vistr.build_and_next();

    if let Some([left, right]) = rest {
        let mut p = splitter.div();

        if node.get_min_elem() <= num_seq_fallback {
            buffer.push(node.finish(sorter));
            left.recurse_seq(splitter, sorter, buffer);
            right.recurse_seq(&mut p, sorter, buffer);
        } else {
            let mut s2 = sorter.div();
            let (_, mut buffer2) = rayon::join(
                || {
                    buffer.push(node.finish(sorter));
                    recurse_par(num_seq_fallback, splitter, sorter, buffer, left);
                },
                || {
                    let num_nodes = num_level::num_nodes(right.get_height());
                    let mut buffer2 = Vec::with_capacity(num_nodes);
                    recurse_par(num_seq_fallback, &mut p, &mut s2, &mut buffer2, right);
                    assert_eq!(num_nodes, buffer2.len());
                    buffer2
                },
            );

            buffer.append(&mut buffer2);
            sorter.add(s2)
        }
        splitter.add(p);
    } else {
        buffer.push(node.finish(sorter));
    }
}
