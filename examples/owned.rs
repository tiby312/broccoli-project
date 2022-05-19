use broccoli::tree::{node::ManySwapBBox, rect};
fn main() {
    let mut acc = [0; 3];

    //Rect is stored directly in tree,
    //but inner is not.
    let aabbs = [
        ManySwapBBox(rect(00, 10, 00, 10), 0),
        ManySwapBBox(rect(15, 20, 15, 20), 1),
        ManySwapBBox(rect(05, 15, 05, 15), 2),
    ];

    //Clones inner into its own vec
    let mut tree = broccoli::TreeOwned::new(aabbs);

    //Find all colliding aabbs.
    tree.as_tree().find_colliding_pairs(|a, b| {
        acc[a.1] += 1;
        acc[b.1] += 1;
    });

    assert_eq!(acc, [1, 1, 2]);
}
