use axgeom::vec2;
use broccoli::{prelude::*,bbox,rect};

fn main() {
    let mut inner1=vec2(5,5);
    let mut inner2=vec2(3,3);
    let mut inner3=vec2(7,7);

    let mut bots = [bbox(rect(0,10,0,10),&mut inner1),
                bbox(rect(2,4,2,4),&mut inner2),
                bbox(rect(6,8,6,8),&mut inner3)];

    let border = broccoli::rect(0, 100, 0, 100);

    let mut tree = broccoli::collections::TreeRef::new(&mut bots);

    let res = tree.k_nearest_mut(
        vec2(30, 30),
        2,
        &mut (),
        |(), a, b| b.distance_squared_to_point(a).unwrap_or(0),
        |(), a, b| b.inner.distance_squared_to_point(a),
        border,
    );
    assert_eq!(res.len(),3);
    assert_eq!(**res[0].as_ref().unwrap().0,vec2(7,7));
    assert_eq!(**res[2].as_ref().unwrap().0,vec2(5,5));

}
