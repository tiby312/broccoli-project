//!
//! The broccoli book mentioned in the root documentation shows that
//! integer comparisons can be faster than floating point.
//!
//! Here are some convinience functions that take a floating point,
//! and then normalize it over an area as integers
//!
//!

use axgeom::Rect;

///Convert a `f32` point to a normalized `u32` point normalized over an area.
#[inline(always)]
#[must_use]
pub fn point_f32_to_u32(a: axgeom::Vec2<f32>, border: &Rect<f32>) -> axgeom::Vec2<u32> {
    axgeom::vec2(convert1d_u32(a.x, border.x), convert1d_u32(a.y, border.y))
}

///Convert a `f32` rect to a normalizde `u32` rect normalized over an area.
#[inline(always)]
#[must_use]
pub fn rect_f32_to_u32(a: Rect<f32>, border: &Rect<f32>) -> Rect<u32> {
    axgeom::rect(
        convert1d_u32(a.x.start, border.x),
        convert1d_u32(a.x.end, border.x),
        convert1d_u32(a.y.start, border.x),
        convert1d_u32(a.y.end, border.x),
    )
}

#[inline(always)]
#[must_use]
fn convert1d_u32(a: f32, range: axgeom::Range<f32>) -> u32 {
    ((a - range.start) * (u32::MAX as f32 / range.distance())) as u32
}

///Convert a `f32` point to a normalized `u32` point normalized over an area.

#[inline(always)]
#[must_use]
pub fn point_f32_to_u16(a: axgeom::Vec2<f32>, border: &Rect<f32>) -> axgeom::Vec2<u16> {
    axgeom::vec2(convert1d_u16(a.x, border.x), convert1d_u16(a.y, border.y))
}

///Convert a `f32` rect to a normalizde `u32` rect normalized over an area.

#[inline(always)]
#[must_use]
pub fn rect_f32_to_u16(a: Rect<f32>, border: &Rect<f32>) -> Rect<u16> {
    axgeom::rect(
        convert1d_u16(a.x.start, border.x),
        convert1d_u16(a.x.end, border.x),
        convert1d_u16(a.y.start, border.x),
        convert1d_u16(a.y.end, border.x),
    )
}

#[inline(always)]
#[must_use]
fn convert1d_u16(a: f32, range: axgeom::Range<f32>) -> u16 {
    ((a - range.start) * (u16::MAX as f32 / range.distance())) as u16
}
