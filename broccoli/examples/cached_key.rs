fn main() {
    let mut inner = [0, 4, 8];

    broccoli::from_cached_key!(tree, &mut inner, |&a| broccoli::rect(a, a + 5, 0, 10));

    tree.find_colliding_pairs(|a, b| {
        broccoli::unpack!(a, b);
        *a += 1;
        *b += 1;
    });

    // bboxes 1st and 2nd intersect, as well as 2nd and 3rd.
    assert_eq!(inner, [0 + 1, 4 + 2, 8 + 1]);
}
