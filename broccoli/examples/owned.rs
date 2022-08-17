use broccoli::rect;

fn main() {
    let mut a = 0;
    let mut b = 0;
    let mut c = 0;

    let mut aabbs = [
        (rect(00, 10, 00, 10), &mut a),
        (rect(15, 20, 15, 20), &mut b),
        (rect(05, 15, 05, 15), &mut c),
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
        **a.unpack_inner() += 1;
        **b.unpack_inner() += 1;
    });

    assert_eq!([a, b, c], [1, 1, 2]);
}
