use broccoli::prelude::CollisionApi;
use broccoli::tree::node::BBox;
use broccoli::tree::Tree;
use broccoli::tree::*;
use broccoli_ext::cachable_pairs::TrustedCollisionPairs;

type MyBBox<'a> = BBox<isize, &'a mut TestNum>;

struct MyTree<'a, 'b>(Tree<'a, MyBBox<'b>>);

unsafe impl<'a, 'b> TrustedCollisionPairs<TestNum> for MyTree<'a, 'b> {
    fn for_every_pair(&mut self, mut func: impl FnMut(&mut TestNum, &mut TestNum)) {
        self.0.colliding_pairs(|a, b| {
            func(a.unpack_inner(), b.unpack_inner());
        })
    }
}

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

    let mut tree = MyTree(broccoli::tree::new(&mut aabbs));

    let mut ctree = broccoli_ext::cachable_pairs::Cacheable::new(&mut tree);

    for _ in 0..100 {
        //Find all colliding aabbs.
        ctree.again(|a, b| {
            a.0 += 1;
            b.0 += 1;
        });
    }

    assert_eq!(a.0, 100);
    assert_eq!(b.0, 201);
    assert_eq!(c.0, 102);
}
