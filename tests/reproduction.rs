#[test]
fn knearest_repro() {
    use axgeom::*;
    let mut repro = [
        (rect(729.75f32, 731.25, -0.75, 0.75), &mut vec2(730.5, 0.)),
        (rect(1517.25, 1518.75, -0.75, 0.75), &mut vec2(1518., 0.)),
    ];

    let mut tree = broccoli::Tree::new(&mut repro);

    let mut res = tree.find_knearest_closure(
        vec2(627.0, 727.5),
        1,
        |point, a| Some(a.0.distance_squared_to_point(point).unwrap_or(0.)),
        |point, a| a.1.distance_squared_to_point(point),
        |point, a| (point.x - a).powi(2),
        |point, a| (point.y - a).powi(2),
    );

    assert_eq!(res.len(), 1);
    assert_eq!(res.total_len(), 1);

    use broccoli::queries::knearest::KnearestResult;
    let r: &[KnearestResult<_>] = res.iter().next().unwrap();
    assert_eq!(r.len(), 1);
}
