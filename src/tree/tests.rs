use super::*;

fn assert_length<I: core::iter::ExactSizeIterator>(it: I) {
    let len = it.size_hint().0;
    assert_eq!(it.count(), len);
}

#[test]
fn test() {
    let mut bots = vec![0usize; 1234];

    let mut bots: Vec<_> = bots
        .iter_mut()
        .map(|a| bbox(axgeom::Rect::new(0isize, 0, 0, 0), a))
        .collect();

    let mut tree = DinoTree::new(&mut bots);
    
    assert!(assert::Assert::tree_invariants(&tree));
    
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
    fn recc(a: VistrMut<NodeMut<BBox<isize, &mut usize>>>) {
        let (_nn, rest) = a.next();
        match rest {
            Some([mut left, mut right]) => {
                {
                    let left = left.create_wrap_mut();
                    let right = right.create_wrap_mut();
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
