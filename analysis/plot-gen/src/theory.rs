use super::*;

fn colfind(man: &mut DnumManager, path: &Path) {
    std::fs::create_dir_all(path.join("colfind")).unwrap();

    for grow in [2.0] {
        let res = colfind::theory(man, 5_000, grow, 1000, 20000);

        let l1 = scatter("brocc", res.iter().map(|(i, r)| (*i as i128, r.brocc)));
        let l2 = scatter("nosort", res.iter().map(|(i, r)| (*i as i128, r.nosort)));
        let l3 = scatter("sweep", res.iter().map(|(i, r)| (*i as i128, r.sweep)));
        let l4 = scatter("naive", res.iter().map(|(i, r)| (*i as i128, r.naive)));

        let m = poloto::build::origin();
        let data = plots!(l1, l2, l3, l4, m);

        let p = simple_fmt!(data, "hay", "x", "y");

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
            let g = res
                .iter()
                .map(move |(grow, levels)| (*grow, levels.rebal[i] as i128));
            poloto::build::line_fill(formatm!("Level {}", i), g)
        })
        .collect();

    let mut file = std::fs::File::create(path.join("level").join("rebal.svg")).unwrap();

    let plot = poloto::simple_fmt!(
        poloto::build::plots_dyn(data).chain(poloto::build::markers([], [0])),
        "rebal",
        "Spiral Grow",
        "Number of Comparisons"
    );

    plot.simple_theme(&mut support::upgrade_write(&mut file))
        .unwrap();
}

pub fn theory(man: &mut DnumManager, path: &Path) {
    colfind(man, path);
    level(man, path);
}
