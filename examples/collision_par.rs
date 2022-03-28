use broccoli::{bbox, rect};
use broccoli::pmut::PMut;
use broccoli::node::BBox;

fn main() {
    let mut inner1 = 0;
    let mut inner2 = 0;
    let mut inner3 = 0;

    //Rect is stored directly in tree,
    //but inner is not.
    let mut aabbs = [
        bbox(rect(0isize, 10, 0, 10), &mut inner1),
        bbox(rect(15, 20, 15, 20), &mut inner2),
        bbox(rect(5, 15, 5, 15), &mut inner3),
    ];

    //This will change the order of the elements
    //in bboxes,but this is okay since we
    //populated it with mutable references.
    let mut tree = broccoli::new(&mut aabbs);

    let func = |a: PMut<BBox<isize, &mut usize>>,
                b: PMut<BBox<isize, &mut usize>>| {
        **a.unpack_inner() += 1;
        **b.unpack_inner() += 1;
    };

    //Find all colliding aabbs.
    let col = tree.colliding_pairs();

    let mut prevec = broccoli::util::PreVec::new();
    let rest = col.next(&mut prevec, func);

    if let Some([left, right]) = rest {
        rayon::join(
            || left.recurse_seq(&mut prevec, func),
            || {
                let mut prevec2 = broccoli::util::PreVec::new();
                right.recurse_seq(&mut prevec2, func)
            },
        );
    }

    assert_eq!(inner1, 1);
    assert_eq!(inner2, 0);
    assert_eq!(inner3, 1);
}
