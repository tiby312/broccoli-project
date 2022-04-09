use super::*;

///A trait that gives the user callbacks at events in a recursive algorithm on the tree.
///The main motivation behind this trait was to track the time spent taken at each level of the tree
///during construction.
pub trait Splitter: Sized {
    ///Called to split this into two to be passed to the children.
    fn div(self) -> (Self, Self);

    ///Called to add the results of the recursive calls on the children.
    fn add(self, b: Self) -> Self;
}

#[must_use]
pub fn build_from_splitter<T: Aabb, S: Sorter, SS: Splitter>(
    tb: impl TreeBuild<T, S>,
    bots: &mut [T],
    splitter: SS,
) -> (TreeInner<Node<T>, S>, SS) {
    pub fn recurse_seq_splitter<'a, T: Aabb, S: Sorter, SS: Splitter>(
        vistr: TreeBuildVisitor<'a, T, S>,
        res: &mut Vec<Node<'a, T>>,
        splitter: SS,
    ) -> SS {
        let NodeBuildResult { node, rest } = vistr.build_and_next();
        res.push(node.finish());
        if let Some([left, right]) = rest {
            let (s1, s2) = splitter.div();

            let s1 = recurse_seq_splitter(left, res, s1);
            let s2 = recurse_seq_splitter(right, res, s2);

            s1.add(s2)
        } else {
            splitter
        }
    }
    let total_num_elem = bots.len();
    let num_level = tb.num_level(bots.len()); //num_level::default(bots.len());
    let mut buffer = Vec::with_capacity(num_level::num_nodes(num_level));
    let vistr = TreeBuildVisitor::new(num_level, bots, tb.sorter());

    let splitter = recurse_seq_splitter(vistr, &mut buffer, splitter);

    let t = TreeInner {
        nodes: buffer,
        sorter: tb.sorter(),
        total_num_elem,
    };
    (t, splitter)
}
#[must_use]
pub fn build_from_splitter_par<T: Aabb, S: Sorter, SS: Splitter + Send>(
    tb: impl TreeBuild<T, S>,
    bots: &mut [T],
    splitter: SS,
) -> (TreeInner<Node<T>, S>, SS)
where
    T: Send + Sync,
    T::Num: Send + Sync,
{
    pub fn recurse_par_splitter<'a, T: Aabb, S: Sorter, SS: Splitter + Send>(
        vistr: TreeBuildVisitor<'a, T, S>,
        height_seq_fallback: usize,
        buffer: &mut Vec<Node<'a, T>>,
        splitter: SS,
    ) -> SS
    where
        T: Send,
        T::Num: Send,
    {
        if vistr.get_height() <= height_seq_fallback {
            vistr.recurse_seq(buffer);
            splitter
        } else {
            let NodeBuildResult { node, rest } = vistr.build_and_next();

            if let Some([left, right]) = rest {
                let (s1, s2) = splitter.div();

                let (s1, (mut a, s2)) = rayon::join(
                    || {
                        buffer.push(node.finish());
                        recurse_par_splitter(left, height_seq_fallback, buffer, s1)
                    },
                    || {
                        let mut f = vec![];
                        let v = recurse_par_splitter(right, height_seq_fallback, &mut f, s2);
                        (f, v)
                    },
                );

                buffer.append(&mut a);
                s1.add(s2)
            } else {
                splitter
            }
        }
    }
    let total_num_elem = bots.len();
    let num_level = tb.num_level(bots.len()); //num_level::default(bots.len());
    let mut buffer = Vec::with_capacity(num_level::num_nodes(num_level));
    let vistr = TreeBuildVisitor::new(num_level, bots, tb.sorter());

    let splitter = recurse_par_splitter(vistr, tb.height_seq_fallback(), &mut buffer, splitter);

    let t = TreeInner {
        nodes: buffer,
        sorter: tb.sorter(),
        total_num_elem,
    };
    (t, splitter)
}
