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

pub fn recurse_seq<'a, T: Aabb, S: Sorter>(
    vistr: TreeBister<'a, T, S>,
    res: &mut Vec<Node<'a, T>>,
) {
    let mut stack = vec![];
    stack.push(vistr);

    while let Some(s) = stack.pop() {
        let (n, rest) = s.build_and_next();
        res.push(n.finish());
        if let Some([left, right]) = rest {
            stack.push(left);
            stack.push(right);
        }
    }
}
