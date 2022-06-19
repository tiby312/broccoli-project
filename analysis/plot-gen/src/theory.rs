use super::*;

fn colfind(man: &mut DnumManager, path: &Path) {
    std::fs::create_dir_all(path.join("colfind")).unwrap();

    for grow in [2.0] {
        let res = colfind::theory(man, 5_000, grow, 1000, 20000);

        let l1 = res
            .iter()
            .map(|(i, r)| (i, r.brocc))
            .cloned_plot()
            .scatter("brocc");
        let l2 = res
            .iter()
            .map(|(i, r)| (i, r.nosort))
            .cloned_plot()
            .scatter("nosort");
        let l3 = res
            .iter()
            .map(|(i, r)| (i, r.sweep))
            .cloned_plot()
            .scatter("sweep");
        let l4 = res
            .iter()
            .map(|(i, r)| (i, r.naive))
            .cloned_plot()
            .scatter("naive");

        let m = poloto::build::origin();

        let p = quick_fmt!("hay", "x", "y", l1, l2, l3, l4, m);

        let mut file =
            std::fs::File::create(path.join("colfind").join(format!("theory_n_{}.svg", grow)))
                .unwrap();

        p.simple_theme(&mut support::upgrade_write(&mut file))
            .unwrap();
    }
}

fn level(man: &mut DnumManager, path: &Path) {
    std::fs::create_dir_all(path.join("level")).unwrap();
    let res = levels::theory(man, 20_000, 0.2, 2.0);

    let num_level = res[0].1.rebal.len();

    let data = (0usize..num_level)
        .map(|i| {
            res.iter()
                .map(move |(grow, levels)| (grow, levels.rebal[i]))
                .cloned_plot()
                .line_fill(formatm!("Level {}", i))
        })
        .collect();

    let mut file = std::fs::File::create(path.join("level").join("rebal.svg")).unwrap();

    let plot = poloto::quick_fmt!(
        "rebal",
        "Spiral Grow",
        "Number of Comparisons",
        poloto::build::plots_dyn(data),
        poloto::build::markers([], [0])
    );

    plot.simple_theme(&mut support::upgrade_write(&mut file))
        .unwrap();
}

pub fn theory(man: &mut DnumManager, path: &Path) {
    colfind(man, path);
    level(man, path);
}
