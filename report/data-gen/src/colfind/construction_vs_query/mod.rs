use super::*;

mod bench;
mod theory;
pub use bench::handle_bench;
pub use theory::handle_theory;

fn repel(p1: Vec2<f32>, p2: Vec2<f32>, res1: &mut Vec2<f32>, res2: &mut Vec2<f32>) {
    let offset = p2 - p1;
    let dis = (offset).magnitude2();
    if dis < RADIUS * RADIUS {
        *res1 += offset * 0.0001;
        *res2 -= offset * 0.0001;
    }
}
