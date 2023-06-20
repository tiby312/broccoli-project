use broccoli::{
    aabb::Aabb,
    aabb::ManySwap,
    build::TreeEmbryo,
    Tree,
    {
        build::Sorter,
        build::{DefaultSorter, NodeBuildResult, TreeBuildVisitor},
    },
};

pub trait RayonBuildPar<'a, T: Aabb> {
    //fn par_new_ext(bots: &'a mut [T], num_level: usize, num_seq_fallback: usize) -> Self;
    fn par_new(bots: &'a mut [T]) -> Self;
}

impl<'a, T: Aabb + ManySwap> RayonBuildPar<'a, T> for Tree<'a, T>
where
    T: Send,
    T::Num: Send,
{
    // fn par_new_ext(bots: &'a mut [T], num_level: usize, num_seq_fallback: usize) -> Self {
    //     assert!(num_level >= 1);
    //     let num_nodes = num_level::num_nodes(num_level);
    //     let mut buffer = Vec::with_capacity(num_nodes);
    //     recurse_par(
    //         num_seq_fallback,
    //         &mut DefaultSorter,
    //         &mut buffer,
    //         TreeBuildVisitor::new(num_level, bots),
    //     );
    //     assert_eq!(buffer.len(), num_nodes);
    //     Tree::from_nodes(buffer)
    // }

    fn par_new(bots: &'a mut [T]) -> Self {
        let (mut buffer, v) = TreeEmbryo::new(bots);
        recurse_par(SEQ_FALLBACK_DEFAULT, &mut DefaultSorter, &mut buffer, v);
        buffer.finish()
    }
}

pub const SEQ_FALLBACK_DEFAULT: usize = 16;

// we want to pass small chunks so that if a slow core
// gets a task, they don't hold everybody else up.

// at the same time, we don't want there to be only
// a few chunks. i.e. only 3 cores available but 4 chunks.

// so lets only result to chunking IF
// the problem size is big enough such that there
// are many chunks.

pub fn recurse_par<'a, T: Aabb + ManySwap, S: Sorter<T> + Clone>(
    num_seq_fallback: usize,
    sorter: &mut S,
    buffer: &mut TreeEmbryo<'a, T, T::Num>,
    vistr: TreeBuildVisitor<'a, T>,
) where
    S: Send,
    T: Send,
    T::Num: Send,
{
    let NodeBuildResult { node, rest } = vistr.build_and_next();

    if let Some([left, right]) = rest {
        if node.get_min_elem() <= num_seq_fallback {
            buffer.add(node.finish(sorter));
            buffer.recurse(left, sorter);
            buffer.recurse(right, sorter);
        } else {
            let mut s2 = sorter.clone();
            let mut b2 = buffer.div();
            rayon::join(
                || {
                    buffer.add(node.finish(sorter));
                    recurse_par(num_seq_fallback, sorter, buffer, left);
                },
                || {
                    recurse_par(num_seq_fallback, &mut s2, &mut b2, right);
                },
            );

            buffer.combine(b2);
        }
    } else {
        buffer.add(node.finish(sorter));
    }
}
