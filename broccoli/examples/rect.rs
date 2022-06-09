use broccoli::tree::{aabb_pin::AabbPin, rect};

fn main() {
    let mut inner1 = 4;
    let mut inner2 = 5;
    let mut inner3 = 6;

    let mut bots = [
        (rect(00, 10, 00, 10), &mut inner1),
        (rect(15, 20, 15, 20), &mut inner2),
        (rect(05, 15, 05, 15), &mut inner3),
    ];

    let mut tree = broccoli::Tree::new(&mut bots);

    let mut rect_collisions = Vec::new();
    tree.find_all_intersect_rect(AabbPin::new(&mut rect(-5, 1, -5, 1)), |_, a| {
        rect_collisions.push(a);
    });

    assert_eq!(rect_collisions.len(), 1);
    assert_eq!(rect_collisions[0].0, rect(0, 10, 0, 10));
    assert_eq!(*rect_collisions[0].1, 4);

    tree.assert_tree_invariants();
}
