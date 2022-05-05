//!
//! Tree building functions that allow arbitrary code to be run every time the problem
//! is split into two and built back together. Useful for debuging/measuring performance.
//!

///A trait that gives the user callbacks at events in a recursive algorithm on the tree.
///The main motivation behind this trait was to track the time spent taken at each level of the tree
///during construction.
pub trait Splitter: Sized {
    ///Called to split this into two to be passed to the children.
    fn div(&mut self) -> Self;

    ///Called to add the results of the recursive calls on the children.
    fn add(&mut self, b: Self);
}

pub type EmptySplitter = [(); 0];

pub fn empty_mut() -> &'static mut EmptySplitter {
    &mut []
}

impl Splitter for [(); 0] {
    fn div(&mut self) -> Self {
        []
    }
    fn add(&mut self, _: Self) {}
}

/*
impl<'a, T: Aabb, S: Sorter<T>> TreeBuilder<'a, T, S> {
    pub fn build_from_splitter<SS: Splitter>(
        self,
        splitter: SS,
    ) -> (TreeInner<Node<'a, T>, S>, SS) {
        pub fn recurse_seq_splitter<'a, T: Aabb, S: Sorter<T>, SS: Splitter>(
            vistr: TreeBuildVisitor<'a, T, S>,
            res: &mut Vec<Node<'a, T>>,
            splitter: SS,
        ) -> SS {
            let NodeBuildResult { node, rest } = vistr.build_and_next();
            if let Some([left, right]) = rest {
                let (s1, s2) = splitter.div();
                res.push(node.finish());

                let s1 = recurse_seq_splitter(left, res, s1);
                let s2 = recurse_seq_splitter(right, res, s2);

                s1.add(s2)
            } else {
                res.push(node.finish());
                splitter
            }
        }
        let TreeBuilder {
            bots,
            sorter,
            num_level,
            ..
        } = self;

        let total_num_elem = bots.len();
        let mut buffer = Vec::with_capacity(num_level::num_nodes(num_level));
        let vistr = TreeBuildVisitor::new(num_level, bots, sorter);

        let splitter = recurse_seq_splitter(vistr, &mut buffer, splitter);

        let t = TreeInner {
            nodes: buffer,
            sorter,
            total_num_elem,
        };
        (t, splitter)
    }

    #[cfg(feature = "parallel")]
    pub fn build_from_splitter_par<SS: Splitter + Send>(
        self,
        splitter: SS,
    ) -> (TreeInner<Node<'a, T>, S>, SS)
    where
        T: Send + Sync,
        T::Num: Send + Sync,
    {
        pub fn recurse_par_splitter<'a, T: Aabb, S: Sorter<T>, SS: Splitter + Send>(
            vistr: TreeBuildVisitor<'a, T, S>,
            num_seq_fallback: usize,
            buffer: &mut Vec<Node<'a, T>>,
            splitter: SS,
        ) -> SS
        where
            T: Send,
            T::Num: Send,
        {
            let NodeBuildResult { node, rest } = vistr.build_and_next();

            if let Some([left, right]) = rest {
                if node.get_num_elem() <= num_seq_fallback {
                    buffer.push(node.finish());
                    left.recurse_seq(buffer);
                    right.recurse_seq(buffer);
                    splitter
                } else {
                    let (s1, s2) = splitter.div();

                    let (s1, (mut a, s2)) = rayon::join(
                        || {
                            buffer.push(node.finish());
                            recurse_par_splitter(left, num_seq_fallback, buffer, s1)
                        },
                        || {
                            let mut f = vec![];
                            let v = recurse_par_splitter(right, num_seq_fallback, &mut f, s2);
                            (f, v)
                        },
                    );

                    buffer.append(&mut a);
                    s1.add(s2)
                }
            } else {
                buffer.push(node.finish());
                splitter
            }
        }
        let TreeBuilder {
            bots,
            sorter,
            num_level,
            num_seq_fallback,
        } = self;

        let total_num_elem = bots.len();
        let mut buffer = Vec::with_capacity(num_level::num_nodes(num_level));
        let vistr = TreeBuildVisitor::new(num_level, bots, sorter);

        let splitter = recurse_par_splitter(vistr, num_seq_fallback, &mut buffer, splitter);

        let t = TreeInner {
            nodes: buffer,
            sorter,
            total_num_elem,
        };
        (t, splitter)
    }
}

*/
