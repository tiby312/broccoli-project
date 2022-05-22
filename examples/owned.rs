use broccoli::tree::{node::ManySwapBBox, rect};
fn main() {
    let mut acc = [0; 3];

    //Rect is stored directly in tree,
    //but inner is not.
    let mut aabbs = [
        ManySwapBBox(rect(00, 10, 00, 10), 0),
        ManySwapBBox(rect(15, 20, 15, 20), 1),
        ManySwapBBox(rect(05, 15, 05, 15), 2),
    ];

    let tree_data = broccoli::Tree::new(&mut aabbs).get_tree_data();

    //Find all colliding aabbs.
    broccoli::Tree::from_tree_data(&mut aabbs, &tree_data).find_colliding_pairs(|a, b| {
        acc[a.1] += 1;
        acc[b.1] += 1;
    });

    assert_eq!(acc, [1, 1, 2]);
}
