use broccoli::aabb::ManySwappable;
use broccoli::tree::rect;
fn main() {
    let mut acc = [0; 3];

    let mut aabbs = [
        ManySwappable((rect(00, 10, 00, 10), 0)),
        ManySwappable((rect(15, 20, 15, 20), 1)),
        ManySwappable((rect(05, 15, 05, 15), 2)),
    ];

    let mut tree = broccoli::Tree::new(&mut aabbs);

    //Find all colliding aabbs.
    tree.find_colliding_pairs(|a, b| {
        let ManySwappable(a) = &*a;
        let ManySwappable(b) = &*b;

        acc[a.1] += 1;
        acc[b.1] += 1;
    });

    assert_eq!(acc, [1, 1, 2]);
}
