use broccoli::rect;
fn main() {
    let mut inner = [0, 4, 8];

    let mut cached = broccoli::Tree::new_by_cached_key(&mut inner, |&a| rect(a, a + 5, 0, 10));
    let mut tree = cached.build();

    // Find all colliding aabbs.
    tree.find_colliding_pairs(|a, b| {
        // We aren't given &mut T reference, but instead of AabbPin<&mut T>.
        // We call unpack_inner() to extract the portion that we are allowed to mutate.
        // (We are not allowed to change the bounding box while in the tree)
        *a.unpack_inner() += 1;
        *b.unpack_inner() += 1;
    });

    // bboxes 1st and 2nd intersect, as well as 2nd and 3rd.
    assert_eq!(inner, [0 + 1, 4 + 2, 8 + 1]);
}
