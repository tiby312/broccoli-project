


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

pub mod build{
    pub fn recurse_seq_splitter<S: Sorter<T>, P: Splitter>(
        self,
        splitter: &mut P,
        sorter: &mut S,
        buffer: &mut Vec<Node<'a, T,T::Num>>
    ) {
        let NodeBuildResult { node, rest } = self.build_and_next();
        buffer.push(node.finish(sorter));
        if let Some([left, right]) = rest {
            let mut a = splitter.div();

            left.recurse_seq(splitter, sorter, buffer);

            right.recurse_seq(&mut a, sorter, buffer);
            splitter.add(a);
        }
    }
}


pub mod query{
    pub mod colfind{
        pub fn recurse_seq_splitter<P: Splitter, N: NodeHandler<T>>(
            self,
            splitter: &mut P,
            func: &mut N,
        ) {
            let (n, rest) = self.collide_and_next(func);

            if let Some([left, right]) = rest {
                let mut s2 = splitter.div();
                n.finish(func);
                left.recurse_seq_splitter( splitter, func);
                right.recurse_seq_splitter( &mut s2, func);
                splitter.add(s2);
            } else {
                n.finish(func);
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
}
