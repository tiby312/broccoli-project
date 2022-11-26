use support::prelude::*;

const MAX: usize = 15_000;

fn test_direct<const K: usize>(grow: f64, val: [u8; K]) -> Vec<(i128, f64, f64)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, val)).take(MAX).collect();
    test_one_kind(&mut all)
}

fn test_indirect<const K: usize>(grow: f64, val: [u8; K]) -> Vec<(i128, f64, f64)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, val)).take(MAX).collect();
    let mut all: Vec<_> = all.iter_mut().collect();
    test_one_kind(&mut all)
}

fn test_default<const K: usize>(grow: f64, val: [u8; K]) -> Vec<(i128, f64, f64)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, val)).take(MAX).collect();
    let mut all: Vec<_> = all.iter_mut().map(|x| Dummy(x.0, &mut x.1)).collect();
    test_one_kind(&mut all)
}

fn test_one_kind<T: ColfindHandler>(all: &mut [T]) -> Vec<(i128, f64, f64)> {
    let mut plots = vec![];
    for a in n_iter(0, MAX) {
        let bots = &mut all[0..a];

        let (mut tree, construct_time) = bench_closure_ret(|| broccoli::Tree::new(bots));

        let (_tree, query_time) = bench_closure_ret(|| {
            tree.find_colliding_pairs(T::handle);
            tree
        });

        plots.push((a as i128, construct_time, query_time));
    }
    plots
}

enum Layout {
    Direct,
    Indirect,
    Default,
}

pub fn bench(emp: &mut Html) -> std::fmt::Result {
    for grow in [0.2, 2.0] {
        for size in [8, 128, 256] {
            let description = formatdoc! {r#"
                Comparison of bench times with elements with {size} bytes. 
                `abspiral(n,{grow})`
            "#};

            let res1 = bench_inner(Layout::Default, grow, size);
            let res2 = bench_inner(Layout::Direct, grow, size);
            let res3 = bench_inner(Layout::Indirect, grow, size);

            let p = plots!(
                plot("c default")
                    .scatter(pcloned(res1.iter().map(|(i, x, _)| (i, x)))),
                plot("c direct")
                    .scatter(pcloned(res2.iter().map(|(i, x, _)| (i, x)))),
                plot("c indirect")
                    .scatter(pcloned(res3.iter().map(|(i, x, _)| (i, x)))),
                plot("q default")
                    .scatter(pcloned(res1.iter().map(|(i, _, x)| (i, x)))),
                plot("q direct")
                    .scatter(pcloned(res2.iter().map(|(i, _, x)| (i, x)))),
                plot("q indirect")
                    .scatter(pcloned(res3.iter().map(|(i, _, x)| (i, x)))),
                poloto::build::origin()
            );

            emp.write_graph(
                Some("layout"),
                &format!("rebal_{}_{}", size, grow),
                "num elements",
                "time taken (seconds)",
                p,
                &description,
            )?;
        }
    }
    Ok(())
}

#[inline(never)]
fn bench_inner(typ: Layout, grow: f64, size: usize) -> Vec<(i128, f64, f64)> {
    match typ {
        Layout::Direct => match size {
            8 => test_direct(grow, [0u8; 8]),
            128 => test_direct(grow, [0u8; 128]),
            256 => test_direct(grow, [0u8; 256]),
            _ => panic!("invalid size"),
        },
        Layout::Indirect => match size {
            8 => test_indirect(grow, [0u8; 8]),
            128 => test_indirect(grow, [0u8; 128]),
            256 => test_indirect(grow, [0u8; 256]),
            _ => panic!("invalid size"),
        },
        Layout::Default => match size {
            8 => test_default(grow, [0u8; 8]),
            128 => test_default(grow, [0u8; 128]),
            256 => test_default(grow, [0u8; 256]),
            _ => panic!("invalid size"),
        },
    }
}
