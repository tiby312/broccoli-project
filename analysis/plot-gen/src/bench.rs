use super::*;

pub fn bench(emp: &mut Html, man: &mut DnumManager) -> std::fmt::Result {
    colfind::theory(emp, man)?;
    colfind::bench(emp)?;
    colfind::bench_grow(emp)?;
    best_height::optimal(emp)?;
    best_height::bench(emp)?;
    cached_pairs::bench(emp)?;
    float_vs_integer::bench(emp)?;
    layout::bench(emp)?;
    par_tuner::bench_par(emp)?;
    par_tuner::best_seq_fallback_rebal(emp)?;
    par_tuner::best_seq_fallback_query(emp)?;
    Ok(())
}
