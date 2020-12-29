extern crate axgeom;
extern crate broccoli;
extern crate compt;

use axgeom::*;
use broccoli::node::*;
use broccoli::prelude::*;
use broccoli::{query::*, Tree};
use compt::*;

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
fn test_tie_knearest() {
    use broccoli::*;

    let mut bots = [
        bbox(rect(5isize, 10, 0, 10), ()),
        bbox(rect(6, 10, 0, 10), ()),
    ];

    let mut tree = broccoli::container::TreeRef::new(&mut bots);

    let mut handler = broccoli::query::knearest::default_rect_knearest(&tree);
    let mut res = tree.k_nearest_mut(vec2(15, 30), 2, &mut handler);

    assert_eq!(res.len(), 2);
    assert_eq!(res.total_len(), 2);

    use broccoli::query::knearest::KnearestResult;
    let r: &[KnearestResult<_>] = res.iter().next().unwrap();
    assert_eq!(r.len(), 2);

    let handler = &mut broccoli::query::knearest::default_rect_knearest(&tree);
    broccoli::query::knearest::assert_k_nearest_mut(&mut tree, vec2(15, 30), 2, handler);
}

#[test]
fn test_tie_raycast() {
    use broccoli::*;
    let mut bots: &mut [BBox<isize, ()>] =
        &mut [bbox(rect(0, 10, 0, 20), ()), bbox(rect(5, 10, 0, 20), ())];

    let mut tree = broccoli::container::TreeRef::new(&mut bots);

    let ray = axgeom::Ray {
        point: vec2(15, 4),
        dir: vec2(-1, 0),
    };

    let mut handler = broccoli::query::raycast::default_rect_raycast(&tree);
    let ans = tree.raycast_mut(ray, &mut handler);

    match ans {
        CastResult::Hit((ans, mag)) => {
            assert_eq!(mag, 5);
            assert_eq!(ans.len(), 2);
        }
        CastResult::NoHit => {
            panic!("should have hit");
        }
    }

    let mut handler = broccoli::query::raycast::default_rect_raycast(&tree);
    broccoli::query::raycast::assert_raycast(&mut tree, ray, &mut handler);
}

#[test]
fn test_zero_sized() {
    let mut bots = vec![(); 1];

    let mut bots = create_bbox_mut(&mut bots, |_b| rect(0isize, 0, 0, 0));

    let tree = broccoli::new(&mut bots);

    let (n, _) = tree.vistr().next();
    assert_eq!(n.div.is_none(), true);
    assert_eq!(n.range.len(), 1);
    assert!(n.cont.is_some());
}

#[test]
fn test_zero_sized2() {
    let mut bots = vec![(); 1];

    let mut bots = create_bbox_mut(&mut bots, |_b| rect(0isize, 0, 0, 0));

    let tree = broccoli::new(&mut bots);

    let (n, _) = tree.vistr().next();
    assert_eq!(n.div.is_none(), true);
    assert_eq!(n.range.len(), 1);
    assert!(n.cont.is_some());
}
#[test]
fn test_one() {
    let mut bots = vec![0usize; 1];

    let mut bots = create_bbox_mut(&mut bots, |_b| rect(0isize, 0, 0, 0));

    let tree = broccoli::new(&mut bots);
    assert_eq!(tree.get_height(), 1);
    let (n, _) = tree.vistr().next();
    assert!(n.div.is_none());
    assert_eq!(n.range.len(), 1);
    assert!(n.cont.is_some())
}

#[test]
fn test_empty() {
    let mut bots: Vec<()> = Vec::new();
    let mut bots = create_bbox_mut(&mut bots, |_b| rect(0isize, 0, 0, 0));
    let tree = broccoli::new(&mut bots);

    assert_eq!(tree.get_height(), 1);

    let (n, _) = tree.vistr().next();
    assert_eq!(n.range.len(), 0);
    assert!(n.div.is_none());
    assert!(n.cont.is_none());
}

#[test]
fn test_many() {
    let mut bots = vec![0usize; 40];

    let mut bots = create_bbox_mut(&mut bots, |_b| rect(0isize, 0, 0, 0));

    let tree = broccoli::new(&mut bots);

    assert_eq!(
        tree.vistr()
            .dfs_inorder_iter()
            .flat_map(|a| a.range.iter())
            .count(),
        40
    );

    let mut num_div = 0;
    for b in tree.vistr().dfs_inorder_iter() {
        if let Some(_) = b.div {
            if let Some(_) = b.cont {
                num_div += 1;
            }
        }
    }
    assert_eq!(num_div, 0);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_send_sync_tree() {
    let mut bots1: Vec<()> = Vec::new();
    let mut bots2: Vec<()> = Vec::new();

    let mut bots1 = create_bbox_mut(&mut bots1, |_| rect(0, 0, 0, 0));
    let mut bots2 = create_bbox_mut(&mut bots2, |_| rect(0, 0, 0, 0));

    //Check that its send
    let (t1, t2) = rayon::join(|| broccoli::new(&mut bots1), || broccoli::new(&mut bots2));

    //Check that its sync
    let (p1, p2) = (&t1, &t2);
    rayon::join(|| p1, || p2);
}
