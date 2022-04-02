#[test]
fn knearest_repro() {
    use axgeom::*;
    use broccoli::prelude::*;
    use broccoli::*;
    let mut repro = [
        bbox(rect(729.75f32, 731.25, -0.75, 0.75), vec2(730.5, 0.)),
        bbox(rect(1517.25, 1518.75, -0.75, 0.75), vec2(1518., 0.)),
    ];

    let mut tree = broccoli::new(&mut repro);

    use broccoli::halfpin::HalfPin;
    use broccoli::node::BBox;
    let mut res = tree.k_nearest_mut_closure(
        vec2(627.0, 727.5),
        1,
        (),
        |_, point, a: HalfPin<&mut BBox<f32, Vec2<f32>>>| {
            Some(a.rect.distance_squared_to_point(point).unwrap_or(0.))
        },
        |_, point, a| a.inner.distance_squared_to_point(point),
        |_, point, a| (point.x - a).powi(2),
        |_, point, a| (point.y - a).powi(2),
    );

    assert_eq!(res.len(), 1);
    assert_eq!(res.total_len(), 1);

    use broccoli::queries::knearest::KnearestResult;
    let r: &[KnearestResult<_>] = res.iter().next().unwrap();
    assert_eq!(r.len(), 1);
}
