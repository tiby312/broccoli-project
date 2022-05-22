use broccoli::tree::{node::ManySwapBBox, rect};
fn main() {
    let mut acc = [0; 3];

    let mut aabbs = [
        ManySwapBBox(rect(00, 10, 00, 10), 0),
        ManySwapBBox(rect(15, 20, 15, 20), 1),
        ManySwapBBox(rect(05, 15, 05, 15), 2),
    ];

    // This is not lifetimed!
    let tree_data = broccoli::Tree::new(&mut aabbs).get_tree_data();

    // Construct the tree again using the pre-computed tree data as
    // as passing in the same elements (in the same order).
    //
    // It is the user's reponsiblity to not reorder the aabbs that
    // were used the make the tree data.
    let mut tree = broccoli::Tree::from_tree_data(&mut aabbs, &tree_data);

    //Find all colliding aabbs.
    tree.find_colliding_pairs(|a, b| {
        acc[a.1] += 1;
        acc[b.1] += 1;
    });

    assert_eq!(acc, [1, 1, 2]);
}
