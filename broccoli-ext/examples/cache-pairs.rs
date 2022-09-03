use broccoli::*;

#[derive(Debug, Copy, Clone)]
struct TestNum(usize);

fn main() {
    let mut a = TestNum(0);
    let mut b = TestNum(1);
    let mut c = TestNum(2);

    let mut aabbs = [
        bbox(rect(0isize, 10, 00, 10), &mut a),
        bbox(rect(0isize, 10, 05, 20), &mut b),
        bbox(rect(0isize, 10, 12, 15), &mut c),
    ];

    let mut tree = broccoli::Tree::new(&mut aabbs);
    let mut tree = broccoli_ext::cacheable_pairs::IndTree(&mut tree);

    let mut ctree = broccoli_ext::cacheable_pairs::CacheSession::new(&mut tree);

    let mut pairs = ctree.cache_colliding_pairs(|_, _| Some(()));

    let mut filtered = ctree.cache_elems(|_| Some(()));

    for _ in 0..100 {
        //Find all colliding aabbs.
        pairs.handle(&mut ctree, |a, b, _| {
            a.0 += 1;
            b.0 += 1;
        });

        for (a, _) in filtered.handle(&mut ctree) {
            a.0 += 1;
        }
    }

    assert_eq!(a.0, 200);
    assert_eq!(b.0, 301);
    assert_eq!(c.0, 202);
}
