use support::{ColfindHandler, Bencher, prelude::Tree};

use support::prelude::*;



#[inline(never)]
pub fn bench(
    max: usize,
    grow: f64,
) -> Vec<(usize, Record)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(max).collect();

    (0..max)
        .step_by(100)
        .map(|a| {
            let bots = &mut all[0..a];
            (
                a,
                new_record(bots),
            )
        })
        .collect()
}


#[derive(Debug)]
pub struct Record {
    pub float: f64,
    pub int: f64,
    pub i64: f64,
    pub float_i32: f64,
}

fn new_record(bots:&mut [Dummy<f32,u32>]) -> Record {
    let mut bencher=Bencher;
    let bench_integer = {
    
        bencher.time(|| {
            let mut tree = Tree::new(bots);

            tree.find_colliding_pairs(Dummy::<f32,u32>::handle);
        })
    };

    let bench_i64 = {
        
        bencher.time(|| {
            let mut tree = broccoli::Tree::new(bots);

            tree.find_colliding_pairs(Dummy::<f32,u32>::handle);
        })
    };

    let bench_float_i32 = {
        
        let border = compute_border(bots).unwrap().inner_as();

        bencher.time(|| {
            let mut bb:Vec<_>=bots.iter().map(|x|Dummy(rect_f32_to_u32(x.0.inner_as(), &border),x.1)).collect();

            let mut tree = broccoli::Tree::new(&mut bb);

            tree.find_colliding_pairs(Dummy::<u32,u32>::handle);
        })
    };

    let bench_float = {
        
        bencher.time(|| {
            let mut tree = broccoli::Tree::new(bots);

            tree.find_colliding_pairs(Dummy::<f32,u32>::handle);
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