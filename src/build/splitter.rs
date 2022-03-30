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

pub fn recurse_seq<'a, T: Aabb, S: Sorter,SS:Splitter>(
    vistr: TreeBister<'a, T, S>,
    res: &mut Vec<Node<'a, T>>,
    mut splitter:SS
)->SS {
    
    let (n,rest)=vistr.build_and_next();
    res.push(n.finish());
    if let Some([left, right]) = rest { 
        let (s1,s2)=splitter.div();

        let s1=recurse_seq(left,res,s1);
        let s2=recurse_seq(right,res,s2);

        splitter.add(s1,s2);
    }

    splitter
}
