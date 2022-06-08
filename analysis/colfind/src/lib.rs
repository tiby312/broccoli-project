use support::prelude::*;

mod bench;
mod theory;

pub use bench::Record as BenchRecord;
pub use theory::Record as TheoryRecord;

pub fn bench_one(num: usize, grow: f64) -> BenchRecord {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(num).collect();
    bench::new_record(&mut all, false, false, false)
}

#[inline(never)]
pub fn bench(
    max: usize,
    grow: f64,
    naive_stop: usize,
    sweep_stop: usize,
) -> Vec<(usize, BenchRecord)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(max).collect();

    (0..max)
        .step_by(100)
        .map(|a| {
            let bots = &mut all[0..a];
            (
                a,
                bench::new_record(bots, true, a < naive_stop, a < sweep_stop),
            )
        })
        .collect()
}

#[inline(never)]
pub fn bench_grow(num: usize, start_grow: f64, end_grow: f64) -> Vec<(f64, BenchRecord)> {
    grow_iter(start_grow, end_grow)
        .map(|grow| {
            let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(num).collect();
            (grow, bench::new_record(&mut all, true, false, true))
        })
        .collect()
}

#[inline(never)]
pub fn theory(
    man: &mut datanum::DnumManager,
    max: usize,
    grow: f64,
    naive_stop: usize,
    sweep_stop: usize,
) -> Vec<(usize, TheoryRecord)> {
    let mut all: Vec<_> = dist::dist_datanum(man, grow)
        .map(|x| Dummy(x, 0u32))
        .take(max)
        .collect();

    (0..max)
        .step_by(100)
        .map(|a| {
            let bots = &mut all[0..a];
            (
                a,
                theory::new_record(man, bots, true, a < naive_stop, a < sweep_stop),
            )
        })
        .collect()
}


#[inline(never)]
pub fn theory_grow(man: &mut datanum::DnumManager,num: usize, start_grow: f64, end_grow: f64) -> Vec<(f64, TheoryRecord)> {
    grow_iter(start_grow, end_grow)
        .map(|grow| {
            let mut all: Vec<_> = dist::dist_datanum(man, grow)
            .map(|x| Dummy(x, 0u32))
            .take(num)
            .collect();
    
            (grow, theory::new_record(man,&mut all,true, false, true))
        })
        .collect()
}

