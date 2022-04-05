use broccoli::tree::*;

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

    let mut tree = broccoli::tree::new(&mut aabbs);

    let mut ctree = broccoli_ext::cachable_pairs::Cacheable::new(&mut tree);

    let mut k=ctree.cache_colliding_pairs(|_,_|{Some(())});

    for _ in 0..100 {
        //Find all colliding aabbs.
        k.colliding_pairs(&mut ctree,|a, b, _| {
            a.0 += 1;
            b.0 += 1;
        });
    }

    assert_eq!(a.0, 100);
    assert_eq!(b.0, 201);
    assert_eq!(c.0, 102);
}
