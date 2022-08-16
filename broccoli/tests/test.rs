use axgeom::*;
use broccoli::aabb::*;
use broccoli::assert::Assert;
use compt::*;

///Convenience function to create a `(Rect<N>,&mut T)` from a `T` and a Rect<N> generating function.
fn create_bbox_mut<'a, N: Num, T>(
    bots: &'a mut [T],
    mut aabb_create: impl FnMut(&T) -> Rect<N>,
) -> Vec<(Rect<N>, &'a mut T)> {
    bots.iter_mut().map(move |k| (aabb_create(k), k)).collect()
}

#[test]
fn test_tie_knearest() {
    let mut bots = [(rect(5isize, 10, 0, 10), ()), (rect(6, 10, 0, 10), ())];

    let mut handler = broccoli::queries::knearest::AabbKnearest;

    Assert::new(&mut bots).assert_k_nearest_mut(vec2(15, 30), 2, &mut handler);

    let mut tree = broccoli::Tree::new(&mut bots);

    let mut res = tree.find_knearest(vec2(15, 30), 2, &mut handler);

    assert_eq!(res.len(), 1);
    assert_eq!(res.total_len(), 2);

    use broccoli::queries::knearest::KnearestResult;
    let r: &[KnearestResult<_>] = res.iter().next().unwrap();
    assert_eq!(r.len(), 2);
}

#[test]
fn test_zero_sized() {
    let mut bots = vec![(); 1];

    let mut bots = create_bbox_mut(&mut bots, |_b| rect(0isize, 0, 0, 0));

    let tree = broccoli::Tree::new(&mut bots);

    let (n, _) = tree.vistr().next();
    assert_eq!(n.div.is_none(), true);
    assert_eq!(n.range.len(), 1);
}

#[test]
fn test_zero_sized2() {
    let mut bots = vec![(); 1];

    let mut bots = create_bbox_mut(&mut bots, |_b| rect(0isize, 0, 0, 0));

    let tree = broccoli::Tree::new(&mut bots);

    let (n, _) = tree.vistr().next();
    assert_eq!(n.div.is_none(), true);
    assert_eq!(n.range.len(), 1);
}
#[test]
fn test_one() {
    let mut bots = vec![0usize; 1];

    let mut bots = create_bbox_mut(&mut bots, |_b| rect(0isize, 0, 0, 0));

    let tree = broccoli::Tree::new(&mut bots);
    assert_eq!(tree.num_levels(), 1);
    let (n, _) = tree.vistr().next();
    assert!(n.div.is_none());
    assert_eq!(n.range.len(), 1);
}

#[test]
fn test_empty() {
    let mut bots: Vec<()> = Vec::new();
    let mut bots = create_bbox_mut(&mut bots, |_b| rect(0isize, 0, 0, 0));
    let tree = broccoli::Tree::new(&mut bots);

    assert_eq!(tree.num_levels(), 1);

    let (n, _) = tree.vistr().next();
    assert_eq!(n.range.len(), 0);
    assert!(n.div.is_none());
}

#[test]
fn test_many() {
    let mut bots = vec![0usize; 40];

    let mut bots = create_bbox_mut(&mut bots, |_b| rect(0isize, 0, 0, 0));

    let tree = broccoli::Tree::new(&mut bots);

    assert_eq!(
        tree.vistr()
            .dfs_preorder_iter()
            .flat_map(|a| a.range.iter())
            .count(),
        40
    );

    let mut num_div = 0;
    for b in tree.vistr().dfs_preorder_iter() {
        if let Some(_) = b.div {
            if !b.range.is_empty() {
                num_div += 1;
            }
        }
    }
    assert_eq!(num_div, 0);
}

// TODO add back?
// #[test]
// #[cfg_attr(miri, ignore)]
// fn test_send_sync_tree() {
//     let mut bots1: Vec<()> = Vec::new();
//     let mut bots2: Vec<()> = Vec::new();

//     let mut bots1 = create_bbox_mut(&mut bots1, |_| rect(0, 0, 0, 0));
//     let mut bots2 = create_bbox_mut(&mut bots2, |_| rect(0, 0, 0, 0));

//     //Check that its send
//     let (t1, t2) = rayon::join(
//         || broccoli::Tree::new(&mut bots1),
//         || broccoli::Tree::new(&mut bots2),
//     );

//     //Check that its sync
//     let (p1, p2) = (&t1, &t2);
//     rayon::join(|| p1, || p2);
// }

#[test]
fn test_tie_raycast() {
    let mut bots: &mut [(Rect<isize>, ())] =
        &mut [(rect(0, 10, 0, 20), ()), (rect(5, 10, 0, 20), ())];

    let mut handler = broccoli::queries::raycast::AabbRaycast;

    let ray = axgeom::Ray {
        point: vec2(15, 4),
        dir: vec2(-1, 0),
    };

    Assert::new(&mut bots).assert_raycast(ray, &mut handler);

    let mut tree = broccoli::Tree::new(&mut bots);

    let ans = tree.cast_ray(ray, &mut handler);

    match ans {
        CastResult::Hit(res) => {
            assert_eq!(res.mag, 5);
            assert_eq!(res.elems.len(), 2);
        }
        CastResult::NoHit => {
            panic!("should have hit");
        }
    }
}
