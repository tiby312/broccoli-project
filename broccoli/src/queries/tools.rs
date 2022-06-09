//! misc tools

use super::*;

#[test]
fn test_section() {
    use axgeom::rect;
    let mut aabbs = [
        rect(1, 4, 0, 0),
        rect(3, 6, 0, 0),
        rect(5, 20, 0, 0),
        rect(6, 50, 0, 0),
        rect(11, 15, 0, 0),
    ];

    let k = get_section_mut(
        axgeom::XAXIS,
        AabbPin::new(&mut aabbs),
        &axgeom::Range::new(5, 10),
    );
    let k: &[axgeom::Rect<isize>] = &k;
    assert_eq!(k.len(), 3);
}

//this can have some false positives.
//but it will still prune a lot of bots.
#[inline(always)]
pub fn get_section_mut<'a, I: Aabb, A: Axis>(
    axis: A,
    arr: AabbPin<&'a mut [I]>,
    range: &Range<I::Num>,
) -> AabbPin<&'a mut [I]> {
    let mut ii = arr.iter().enumerate();

    let ii = &mut ii;

    let start = {
        let mut ii = ii.skip_while(|(_, i)| {
            let rr = i.get().get_range(axis);
            rr.end < range.start
        });

        if let Some(start) = ii.next() {
            start.0
        } else {
            return AabbPin::new(&mut []);
        }
    };

    let end = {
        let mut ii = ii.skip_while(|(_, i)| {
            let rr = i.get().get_range(axis);
            rr.start <= range.end
        });

        if let Some((end, _)) = ii.next() {
            end
        } else {
            return arr.truncate_from(start..);
        }
    };

    arr.truncate(start..end)
}
