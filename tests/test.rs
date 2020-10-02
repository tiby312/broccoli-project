extern crate axgeom;
extern crate compt;
extern crate dinotree_alg;

use compt::*;

use axgeom::*;
use dinotree_alg::par::*;
use dinotree_alg::*;

///Convenience function to create a `(Rect<N>,&mut T)` from a `T` and a Rect<N> generating function.
fn create_bbox_mut<'a, N: Num, T>(
    bots: &'a mut [T],
    mut aabb_create: impl FnMut(&T) -> Rect<N>,
) -> Vec<BBox<N, &'a mut T>> {
    bots.iter_mut()
        .map(move |k| BBox::new(aabb_create(k), k))
        .collect()
}

#[test]
fn test_par_heur() {
    let p = compute_level_switch_sequential(6, 6);
    assert_eq!(p.get_depth_to_switch_at(), 0);
}
#[test]
fn test_parallel() {
    let k = Parallel::new(0);
    match k.next() {
        ParResult::Parallel(_) => {
            panic!("fail");
        }
        ParResult::Sequential(_) => {}
    }
}

#[test]
fn test_zero_sized() {
    let mut bots = vec![(); 1];

    let mut bots = create_bbox_mut(&mut bots, |_b| axgeom::Rect::new(0isize, 0, 0, 0));

    let tree = DinoTree::new(&mut bots);

    let (n, _) = tree.vistr().next();
    let n = n.get();
    assert_eq!(n.div.is_none(), true);
    assert_eq!(n.bots.len(), 1);
    assert!(n.cont.is_some());
}

#[test]
fn test_zero_sized2() {
    let mut bots = vec![(); 1];

    let mut bots = create_bbox_mut(&mut bots, |_b| axgeom::Rect::new(0isize, 0, 0, 0));

    let tree = DinoTree::new(&mut bots);

    let (n, _) = tree.vistr().next();
    let n = n.get();
    assert_eq!(n.div.is_none(), true);
    assert_eq!(n.bots.len(), 1);
    assert!(n.cont.is_some());
}
#[test]
fn test_one() {
    let mut bots = vec![0usize; 1];

    let mut bots = create_bbox_mut(&mut bots, |_b| axgeom::Rect::new(0isize, 0, 0, 0));

    let tree = DinoTree::new(&mut bots);

    let (n, _) = tree.vistr().next();
    let n = n.get();
    assert!(n.div.is_none());
    assert_eq!(n.bots.len(), 1);
    assert!(n.cont.is_some())
}

#[test]
fn test_empty() {
    let mut bots: Vec<()> = Vec::new();
    let mut bots = create_bbox_mut(&mut bots, |_b| axgeom::Rect::new(0isize, 0, 0, 0));
    let tree = DinoTree::new(&mut bots);

    let (n, _) = tree.vistr().next();
    let n = n.get();
    assert_eq!(n.bots.len(), 0);
    assert!(n.div.is_none());
    assert!(n.cont.is_none());
}

#[test]
fn test_many() {
    let mut bots = vec![0usize; 1000];

    let mut bots = create_bbox_mut(&mut bots, |_b| axgeom::Rect::new(0isize, 0, 0, 0));

    let tree = DinoTree::new(&mut bots);

    assert_eq!(
        tree.vistr()
            .dfs_inorder_iter()
            .flat_map(|a| a.get().bots.iter())
            .count(),
        1000
    );

    let mut num_div = 0;
    for b in tree.vistr().dfs_inorder_iter() {
        if let Some(_) = b.get().div {
            if let Some(_) = b.get().cont {
                num_div += 1;
            }
        }
    }
    assert_eq!(num_div, 0);
}

#[test]
fn test_send_sync_dinotree() {
    let mut bots1: Vec<()> = Vec::new();
    let mut bots2: Vec<()> = Vec::new();

    let mut bots1 = create_bbox_mut(&mut bots1, |_| axgeom::Rect::new(0, 0, 0, 0));
    let mut bots2 = create_bbox_mut(&mut bots2, |_| axgeom::Rect::new(0, 0, 0, 0));

    //Check that its send
    let (t1, t2) = rayon::join(|| DinoTree::new(&mut bots1), || DinoTree::new(&mut bots2));

    //Check that its sync
    let (p1, p2) = (&t1, &t2);
    rayon::join(|| p1, || p2);
}
