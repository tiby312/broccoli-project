use broccoli::tree::rect;
fn main() {
    let mut inner1 = 0;
    let mut inner2 = 0;
    let mut inner3 = 0;

    let mut aabbs = [
        (rect(00, 10, 00, 10), &mut inner1),
        (rect(15, 20, 15, 20), &mut inner2),
        (rect(05, 15, 05, 15), &mut inner3),
    ];

    // Construct tree by doing many swapping of elements
    let tree = broccoli::Tree::new(&mut aabbs);

    // Store tree data so we can rebuild it.
    let data = tree.get_tree_data();

    // Tree gets destroyed and access the aabbs directly.
    // It is important that the user does not swap elements around
    // or change their aabbs.
    for a in aabbs.iter() {
        dbg!(&a.0);
    }

    // Rebuild the tree using the stored tree data.
    let mut tree = broccoli::Tree::from_tree_data(&mut aabbs, &data);

    // Find all colliding aabbs.
    tree.find_colliding_pairs(|a, b| {
        // We aren't given &mut T reference, but instead of AabbPin<&mut T>.
        // We call unpack_inner() to extract the portion that we are allowed to mutate.
        // (We are not allowed to change the bounding box while in the tree)
        **a.unpack_inner() += 1;
        **b.unpack_inner() += 1;
    });

    assert_eq!(inner1, 1);
    assert_eq!(inner2, 1);
    assert_eq!(inner3, 2);
}
