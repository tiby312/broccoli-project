use broccoli::{tree::{node::{Aabb, ManySwap, Node}, BuildArgs, build::{DefaultSorter, TreeBuildVisitor, NodeBuildResult}, splitter::{Splitter, EmptySplitter}, Sorter}, Tree};

use broccoli::tree::num_level;




pub struct ParBuildArgs{
    num_seq_fallback:usize
}

impl ParBuildArgs{
    pub fn new()->ParBuildArgs{
        ParBuildArgs { num_seq_fallback: 512 }
    }
}


pub struct ParBuilder<'a,T>{
    elem:&'a mut [T]
}
impl<'a,T:Aabb+ManySwap+Send> ParBuilder<'a,T> where T::Num:Send{
    pub fn new(elem:&'a mut [T])->Self{
        ParBuilder{
            elem
        }
    }
}
impl<'a,T:Aabb+ManySwap+Send> ParBuilder<'a,T> where T::Num:Send{
    pub fn par_build(self) -> Tree<'a, T>
    {
        let bots=self.elem;
        let (nodes, _) = par_build_ext(bots, &mut DefaultSorter,BuildArgs::new(bots.len()),ParBuildArgs::new(),EmptySplitter);
    
        Tree::from_nodes(nodes)
    }

        
    pub fn par_build_from_args<P: Splitter>(bots: &'a mut [T],splitter:P, args1:BuildArgs,args: ParBuildArgs) -> (Tree<'a, T>, P)
    where    P: Send,
    {
        let (nodes, s) = par_build_ext(bots, &mut DefaultSorter,args1,args,splitter);
        (Tree::from_nodes(nodes),s)
    }

}


pub fn par_build_ext<'a, T: Aabb + ManySwap, S,P:Splitter>(
    bots: &'a mut [T],
    sorter: &mut S,
    args:BuildArgs,
    par_args:ParBuildArgs,
    mut splitter:P
) -> (Vec<Node<'a, T>>, P)
where
    S: Sorter<T>,
    T: Send,
    T::Num: Send,
    S: Send,
    P: Splitter + Send,
{
    let mut buffer = Vec::with_capacity(num_level::num_nodes(args.num_level));
    recurse_par(
        par_args.num_seq_fallback,
        &mut splitter,
        sorter,
        &mut buffer,
        TreeBuildVisitor::new(args.num_level, bots),
    );
    (buffer, splitter)
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
            //dbg!(node.get_num_elem());
            let (_, mut buffer2) = rayon::join(
                || {
                    buffer.push(node.finish(sorter));
                    recurse_par(num_seq_fallback, splitter, sorter, buffer, left);
                },
                || {
                    let mut buffer2 = Vec::with_capacity(num_level::num_nodes(right.get_height()));

                    recurse_par(num_seq_fallback, &mut p, &mut s2, &mut buffer2, right);
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
