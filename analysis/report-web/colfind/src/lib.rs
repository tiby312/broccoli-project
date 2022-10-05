use support::prelude::*;

mod bench;
mod theory;

use poloto::build::plot;

pub use bench::Record as BenchRecord;
pub use theory::Record as TheoryRecord;

pub fn bench_one(num: usize, grow: f64) -> BenchRecord {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(num).collect();
    bench::new_record(&mut all, false, false, false)
}

pub fn theory(emp: &mut Html, man: &mut DnumManager) -> std::fmt::Result {
    for grow in [0.5, 2.0] {
        let description = formatdoc! {r#"
            Comparison of theory times of different collision finding strategies. 
            `abspiral(n,{grow})`
        "#};

        let res = theory_inner(man, 5000, grow, 1500, 2000);

        let l1 = plot("brocc")
            .scatter()
            .cloned(res.iter().map(|(i, r)| (*i, r.brocc)));
        let l2 = plot("naive")
            .scatter()
            .cloned(res.iter().map(|(i, r)| (*i, r.naive)));
        let l3 = plot("sweep")
            .scatter()
            .cloned(res.iter().map(|(i, r)| (*i, r.sweep)));
        let l4 = plot("nosort")
            .scatter()
            .cloned(res.iter().map(|(i, r)| (*i, r.nosort)));

        let m = poloto::build::origin();

        emp.write_graph(
            Some("theory_colfind"),
            &format!("n_{}", grow),
            "num elements",
            "time taken (seconds)",
            plots!(l1, l2, l3, l4, m),
            &description,
        )?;
    }
    Ok(())
}

pub fn theory_grow(emp: &mut Html, man: &mut DnumManager) -> std::fmt::Result {
    let n = 5000;

    let description = formatdoc! {r#"
        num comparison of different collision finding strategies. 
        `abspiral({n},x)`
    "#};

    let res = theory_grow_inner(man, n, 0.2, 1.5);

    let p = plots!(
        plot("brocc")
            .scatter()
            .cloned(res.iter().map(|(x, i)| (*x, i.brocc))),
        plot("nosort")
            .scatter()
            .cloned(res.iter().map(|(x, i)| (*x, i.nosort))),
        plot("sweep")
            .scatter()
            .cloned(res.iter().map(|(x, i)| (*x, i.sweep))),
        plot("naive")
            .scatter()
            .cloned(res.iter().map(|(x, i)| (*x, i.naive)))
    );

    emp.write_graph(
        Some("colfind"),
        &format!("grow_{}", n),
        "grow",
        "num comparison",
        p,
        &description,
    )
}

pub fn bench_grow(emp: &mut Html) -> std::fmt::Result {
    let n = 10_000;

    let description = formatdoc! {r#"
            Comparison of bench times of different collision finding strategies. 
            `abspiral({n},x)`
        "#};

    let res = bench_grow_inner(n, 0.2, 1.5);

    let p = plots!(
        plot("brocc")
            .scatter()
            .cloned(res.iter().map(|(i, r)| (i, r.brocc))),
        plot("brocc_par")
            .scatter()
            .cloned(res.iter().map(|(i, r)| (i, r.brocc_par))),
        plot("nosort")
            .scatter()
            .cloned(res.iter().map(|(i, r)| (i, r.nosort))),
        plot("nosort_par")
            .scatter()
            .cloned(res.iter().map(|(i, r)| (i, r.nosort_par))),
        plot("sweep")
            .scatter()
            .cloned(res.iter().map(|(i, r)| (i, r.sweep))),
        plot("naive")
            .scatter()
            .cloned(res.iter().map(|(i, r)| (i, r.naive))),
        poloto::build::markers([], [0.0])
    );

    emp.write_graph(
        Some("colfind"),
        &format!("grow_{}", n),
        "grow",
        "time taken (seconds)",
        p,
        &description,
    )
}

pub fn bench(emp: &mut Html) -> std::fmt::Result {
    for (grow, n) in [(0.5, 8_000), (2.0, 10_000)] {
        let description = formatdoc! {r#"
            Comparison of bench times of different collision finding strategies. 
            `abspiral(n,{grow})`
        "#};

        let res = self::bench_inner(n, grow, 2000, 20000);

        
        let p = plots!(
            plot("brocc")
                .scatter()
                .cloned(res.iter().map(|(i, r)| (i, r.brocc))),
            plot("brocc_par")
                .scatter()
                .cloned(res.iter().map(|(i, r)| (i, r.brocc_par))),
            plot("nosort")
                .scatter()
                .cloned(res.iter().map(|(i, r)| (i, r.nosort))),
            plot("nosort_par")
                .scatter()
                .cloned(res.iter().map(|(i, r)| (i, r.nosort_par))),
            plot("sweep")
                .scatter()
                .cloned(res.iter().map(|(i, r)| (i, r.sweep))),
            plot("naive")
                .scatter()
                .cloned(res.iter().map(|(i, r)| (i, r.naive))),
            poloto::build::origin()
        );

        let group_name = "colfind";
        let name = &format!("n_{}", grow);

        emp.write_graph(
            Some(group_name),
            name,
            "num elements",
            "time taken (seconds)",
            p,
            &description,
        )?;
    }

    Ok(())
}

#[inline(never)]
fn theory_inner(
    man: &mut datanum::DnumManager,
    max: usize,
    grow: f64,
    naive_stop: usize,
    sweep_stop: usize,
) -> Vec<(i128, TheoryRecord)> {
    let mut all: Vec<_> = dist::dist_datanum(man, grow)
        .map(|x| Dummy(x, 0u32))
        .take(max)
        .collect();

    (0..max)
        .step_by(100)
        .map(|a| {
            let bots = &mut all[0..a];
            (
                a as i128,
                theory::new_record(man, bots, true, a < naive_stop, a < sweep_stop),
            )
        })
        .collect()
}

#[inline(never)]
fn bench_inner(
    max: usize,
    grow: f64,
    naive_stop: usize,
    sweep_stop: usize,
) -> Vec<(i128, BenchRecord)> {
    let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(max).collect();

    (0..max)
        .step_by(100)
        .map(|a| {
            let bots = &mut all[0..a];
            (
                a as i128,
                bench::new_record(bots, true, a < naive_stop, a < sweep_stop),
            )
        })
        .collect()
}

#[inline(never)]
fn bench_grow_inner(num: usize, start_grow: f64, end_grow: f64) -> Vec<(f64, BenchRecord)> {
    grow_iter(start_grow, end_grow)
        .map(|grow| {
            let mut all: Vec<_> = dist::dist(grow).map(|x| Dummy(x, 0u32)).take(num).collect();
            (grow, bench::new_record(&mut all, true, false, true))
        })
        .collect()
}

#[inline(never)]
fn theory_grow_inner(
    man: &mut datanum::DnumManager,
    num: usize,
    start_grow: f64,
    end_grow: f64,
) -> Vec<(f64, TheoryRecord)> {
    grow_iter(start_grow, end_grow)
        .map(|grow| {
            let mut all: Vec<_> = dist::dist_datanum(man, grow)
                .map(|x| Dummy(x, 0u32))
                .take(num)
                .collect();

            (grow, theory::new_record(man, &mut all, true, false, true))
        })
        .collect()
}
