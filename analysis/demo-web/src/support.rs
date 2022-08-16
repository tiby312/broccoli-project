use super::*;
use axgeom::*;

fn rand() -> f32 {
    js_sys::Math::random() as f32
}

pub fn make_rand(border: Rect<f32>) -> impl Iterator<Item = [f32; 2]> + Clone + Send + Sync {
    std::iter::repeat_with(move || {
        let randx = rand();
        let randy = rand();

        let xx = border.x.start + randx * border.x.end;
        let yy = border.y.start + randy * border.y.end;
        [xx, yy]
    })
}

pub fn make_rand_rect(
    border: Rect<f32>,
    radius: [f32; 2],
) -> impl Iterator<Item = Rect<f32>> + Clone + Send + Sync {
    std::iter::repeat_with(move || {
        let randx = rand();
        let randy = rand();
        let radiusr = rand();

        let xx = border.x.start + randx * border.x.end;
        let yy = border.y.start + randy * border.y.end;
        let radius = radius[0] + (radius[1] - radius[0]) * radiusr;
        Rect::from_point(vec2(xx, yy), vec2same(radius))
    })
}

use broccoli::*;
pub fn point_to_rect_f32(a: axgeom::Vec2<f32>, radius: f32) -> Rect<f32> {
    Rect::from_point(a, axgeom::vec2same(radius))
}

pub fn distribute<X, T: Num>(
    inner: &mut [X],
    mut func: impl FnMut(&X) -> Rect<T>,
) -> Vec<(Rect<T>, &mut X)> {
    inner.iter_mut().map(|a| (func(a), a)).collect()
}
