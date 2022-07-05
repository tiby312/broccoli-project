use support::{prelude::Tree, Bencher, ColfindHandler};

use support::prelude::*;


pub fn bench(emp:&mut Html)->std::fmt::Result{

    
        let grow = 2.0;
        let description = formatdoc! {r#"
            Comparison of bench times using different number types as problem
            size increases. `abspiral(n,{grow})`
        "#};

        let res = bench_inner(10_000, 2.0);
        let l1 = res
            .iter()
            .map(|(i, r)| (i, r.float))
            .cloned_plot()
            .scatter("f32");
        let l2 = res
            .iter()
            .map(|(i, r)| (i, r.int))
            .cloned_plot()
            .scatter("i32");
        let l3 = res
            .iter()
            .map(|(i, r)| (i, r.i64))
            .cloned_plot()
            .scatter("i64");
        let l4 = res
            .iter()
            .map(|(i, r)| (i, r.float_i32))
            .cloned_plot()
            .scatter("f32->int");

        let m = poloto::build::origin();

        emp.write_graph(
            None,
            "float-int",
            "num elements",
            "time taken (seconds)",
            plots!(l1, l2, l3, l4, m),
            &description,
        )
    

}
#[inline(never)]
fn bench_inner(max: usize, grow: f64) -> Vec<(i128, Record)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(max).collect();

    (0..max)
        .step_by(100)
        .skip(1)
        .map(|a| {
            let bots = &mut all[0..a];
            (a as i128, new_record(bots))
        })
        .collect()
}

#[derive(Debug)]
struct Record {
    pub float: f64,
    pub int: f64,
    pub i64: f64,
    pub float_i32: f64,
}

fn new_record(bots: &mut [Dummy<f32, u32>]) -> Record {
    assert!(!bots.is_empty());

    let mut bencher = Bencher;
    let bench_integer = {
        bencher.time(|| {
            let mut tree = Tree::new(bots);

            tree.find_colliding_pairs(Dummy::<f32, u32>::handle);
        })
    };

    let bench_i64 = {
        bencher.time(|| {
            let mut tree = broccoli::Tree::new(bots);

            tree.find_colliding_pairs(Dummy::<f32, u32>::handle);
        })
    };

    let bench_float_i32 = {
        let border = compute_border(bots).unwrap().inner_as();

        bencher.time(|| {
            let mut bb: Vec<_> = bots
                .iter()
                .map(|x| Dummy(rect_f32_to_u32(x.0.inner_as(), &border), x.1))
                .collect();

            let mut tree = broccoli::Tree::new(&mut bb);

            tree.find_colliding_pairs(Dummy::<u32, u32>::handle);
        })
    };

    let bench_float = {
        bencher.time(|| {
            let mut tree = broccoli::Tree::new(bots);

            tree.find_colliding_pairs(Dummy::<f32, u32>::handle);
        })
    };

    Record {
        i64: bench_i64 as f64,
        float: bench_float as f64,
        int: bench_integer as f64,
        float_i32: bench_float_i32 as f64,
    }
}

fn compute_border<T: Aabb>(bb: &[T]) -> Option<Rect<T::Num>> {
    let (first, rest) = bb.split_first()?;
    let mut r = *first.get();
    for a in rest.iter() {
        r.grow_to_fit(a.get());
    }
    Some(r)
}

///Convert a `f32` rect to a normalizde `u32` rect normalized over an area.
#[inline(always)]
fn rect_f32_to_u32(a: Rect<f32>, border: &Rect<f32>) -> Rect<u32> {
    axgeom::rect(
        convert1d_u32(a.x.start, border.x),
        convert1d_u32(a.x.end, border.x),
        convert1d_u32(a.y.start, border.y),
        convert1d_u32(a.y.end, border.y),
    )
}

#[inline(always)]
fn convert1d_u32(a: f32, range: axgeom::Range<f32>) -> u32 {
    ((a - range.start) * (u32::MAX as f32 / range.distance())) as u32
}
