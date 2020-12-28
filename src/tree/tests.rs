use super::*;

fn assert_length<I: core::iter::ExactSizeIterator>(it: I) {
    let len = it.size_hint().0;
    assert_eq!(it.count(), len);
}

#[test]
fn test() {
    let mut bots = vec![0usize; 64];

    let mut bots: Vec<_> = bots
        .iter_mut()
        .map(|a| crate::bbox(rect(0isize, 0, 0, 0), a))
        .collect();
    use crate::query::NaiveComparable;
    let mut tree = crate::container::TreeRef::new(&mut bots);

    assert!(tree.assert_tree_invariants());

    assert_length(tree.vistr_mut().dfs_preorder_iter());
    assert_length(tree.vistr().dfs_preorder_iter());

    let num_nodes = tree.num_nodes();

    assert_eq!(
        tree.vistr_mut().dfs_preorder_iter().size_hint().0,
        num_nodes
    );

    assert_eq!(tree.vistr().dfs_preorder_iter().size_hint().0, num_nodes);

    recc(tree.vistr_mut());
    //recursively check that the length is correct at each node.
    fn recc(a: VistrMut<Node<BBox<isize, &mut usize>>>) {
        let (_nn, rest) = a.next();
        match rest {
            Some([mut left, mut right]) => {
                {
                    let left = left.borrow_mut();
                    let right = right.borrow_mut();
                    assert_length(left.dfs_preorder_iter());
                    assert_length(right.dfs_preorder_iter());
                }
                recc(left);
                recc(right);
            }
            None => {}
        }
    }
}
