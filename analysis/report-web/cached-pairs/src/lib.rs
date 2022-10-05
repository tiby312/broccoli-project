use support::prelude::*;

pub fn bench(emp: &mut Html) -> std::fmt::Result {
    let num = 10_000;
    let grow = 1.0;
    let num_iter = 2;

    let description = formatdoc! {r#"
            Query vs Cached Query with {num_iter} iterations of `abspiral(num,{grow})`.
        "#};
    let res = bench_inner(num, grow, num_iter);

    let a = plot("no cache")
        .scatter()
        .cloned(res.iter().map(|(x, y)| (*x, y.bench)));

    let b = plot("cached")
        .scatter()
        .cloned(res.iter().map(|(x, y)| (*x, y.collect)));

    emp.write_graph(
        None,
        "collect",
        "num elements",
        "time taken (seconds)",
        plots!(a, b),
        &description,
    )
}
#[derive(Debug)]
struct Res {
    pub bench: f64,
    pub collect: f64,
}

#[inline(never)]
fn bench_inner(max: usize, grow: f64, num_iter: usize) -> Vec<(i128, Res)> {
    assert!(num_iter >= 1);
    let mut bencher = Bencher;
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(max).collect();

    (0..max)
        .step_by(100)
        .map(|n| {
            let bots = &mut all[0..n];

            let control = bencher.time(|| {
                let mut t = Tree::new(bots);
                for _ in 0..num_iter {
                    t.find_colliding_pairs(Dummy::<f32, u32>::handle);
                }
            });

            let test = bencher.time(|| {
                let mut tree = Tree::new(bots);
                let mut tree = broccoli_ext::cacheable_pairs::IndTree(&mut tree);
                let mut cacher = broccoli_ext::cacheable_pairs::CacheSession::new(&mut tree);
                let mut pairs = cacher.cache_colliding_pairs(|a, b| {
                    *a ^= 1;
                    *b ^= 1;
                    Some(())
                });

                for _ in 1..num_iter {
                    pairs.handle(&mut cacher, |a, b, ()| {
                        *a ^= 1;
                        *b ^= 1;
                    });
                }
            });

            (
                n as i128,
                Res {
                    bench: control,
                    collect: test,
                },
            )
        })
        .collect()
}
