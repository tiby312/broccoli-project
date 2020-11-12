use broccoli::prelude::*;
use axgeom::vec2;
use broccoli::{Rect,convert::rect_f32_to_u32};

///This showcases making normalized integer aabbs from floats and a specified border.
fn main() {
    let border=broccoli::rect(-100.0,100.0,-100.0,100.0);
    let radius=vec2(5.0,5.0);

    
    let mut inner1 = (    vec2( 0.0, 0.0)     ,0);
    let mut inner2 = (    vec2(20.0,20.0)     ,0);
    let mut inner3 = (    vec2( 4.0, 4.0)     ,0);

    //Using a semi-direct layout for best performance.
    //rect is stored directly in tree, but inner is not.
    let mut aabbs = [
        broccoli::bbox(rect_f32_to_u32(Rect::from_point(inner1.0,radius),&border), &mut inner1),
        broccoli::bbox(rect_f32_to_u32(Rect::from_point(inner2.0,radius),&border), &mut inner2),
        broccoli::bbox(rect_f32_to_u32(Rect::from_point(inner3.0,radius),&border), &mut inner3),
    ];

    //This will change the order of the elements in bboxes,
    //but this is okay since we populated it with mutable references.
    let mut tree = broccoli::new(&mut aabbs);

    //Find all colliding aabbs.
    tree.find_colliding_pairs_mut(|a, b| {
        let offset=a.0-b.0;
        let dis=offset.magnitude2();
        a.1+=dis as usize;
        b.1+=dis as usize;
    });

    assert_eq!(inner1.1, 5);
    assert_eq!(inner2.1, 0);
    assert_eq!(inner3.1, 5);
}
