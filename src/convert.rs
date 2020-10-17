//!
//! The broccoli book mentioned in the root documentation shows that
//! integer comparisons can be faster than floating point.
//!
//! Here are some convinience functions that take a floating point,
//! and then normalize it over an area as integers
//!
//!

use axgeom::Rect;

pub fn point_f32_to_u32(a:axgeom::Vec2<f32>,border:&Rect<f32>)->axgeom::Vec2<u32>{
    axgeom::vec2(convert1d(a.x,border.x),convert1d(a.y,border.y))
}

///Helper function
pub fn rect_f32_to_u32(a:Rect<f32>,border:&Rect<f32>)->Rect<u32>{
    axgeom::rect(
        convert1d(a.x.start,border.x),
        convert1d(a.x.end,border.x),
        convert1d(a.y.start,border.x),
        convert1d(a.y.end,border.x)
    )
}

fn convert1d(a:f32,range:axgeom::Range<f32>)->u32{
    ((a-range.start) * (u32::MAX as f32 /range.distance())) as u32
}

