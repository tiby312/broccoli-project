use super::*;

///A trait that gives the user callbacks at events in a recursive algorithm on the tree.
///The main motivation behind this trait was to track the time spent taken at each level of the tree
///during construction.
pub trait Splitter: Sized {
    ///Called to split this into two to be passed to the children.
    fn div(&mut self) -> (Self, Self);

    ///Called to add the results of the recursive calls on the children.
    fn add(&mut self, a: Self, b: Self);
}

pub fn recurse_seq_splitter<'a, T: Aabb, S: Sorter, SS: Splitter>(
    vistr: TreeBister<'a, T, S>,
    res: &mut Vec<Node<'a, T>>,
    mut splitter: SS,
) -> SS {
    let Res { node, rest } = vistr.build_and_next();
    res.push(node.finish());
    if let Some([left, right]) = rest {
        let (s1, s2) = splitter.div();

        let s1 = recurse_seq_splitter(left, res, s1);
        let s2 = recurse_seq_splitter(right, res, s2);

        splitter.add(s1, s2);
    }

    splitter
}

pub fn recurse_par_splitter<'a, T: Aabb, S: Sorter, SS: Splitter + Send>(
    vistr: TreeBister<'a, T, S>,
    height_seq_fallback: usize,
    buffer: &mut Vec<Node<'a, T>>,
    mut splitter: SS,
) -> SS
where
    T: Send,
    T::Num: Send,
{
    if vistr.get_height() <= height_seq_fallback {
        vistr.recurse_seq(buffer);
    } else {
        let Res { node, rest } = vistr.build_and_next();

        if let Some([left, right]) = rest {
            let (s1, s2) = splitter.div();

            let (s1, (mut a, s2)) = rayon_core::join(
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
            splitter.add(s1, s2);

            buffer.append(&mut a);
        }
    }

    splitter
}
