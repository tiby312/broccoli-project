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
        TreePin::new(&mut aabbs),
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
    arr: TreePin<&'a mut [I]>,
    range: &Range<I::Num>,
) -> TreePin<&'a mut [I]> {
    let mut start = None;
    let mut ii = arr.iter().enumerate();
    for (e, i) in &mut ii {
        let rr = i.get().get_range(axis);
        if rr.end >= range.start {
            start = Some(e);
            break;
        }
    }

    let start = if let Some(start) = start {
        start
    } else {
        return TreePin::new(&mut []);
    };

    let mut end = None;
    for (e, i) in ii {
        let rr = i.get().get_range(axis);
        if rr.start > range.end {
            end = Some(e);
            break;
        }
    }

    if let Some(end) = end {
        arr.truncate(start..end)
    } else {
        arr.truncate_from(start..)
    }
}

pub fn for_every_pair<T: Aabb>(
    mut arr: TreePin<&mut [T]>,
    mut func: impl FnMut(TreePin<&mut T>, TreePin<&mut T>),
) {
    loop {
        let temp = arr;
        match temp.split_first_mut() {
            Some((mut b1, mut x)) => {
                for mut b2 in x.borrow_mut().iter_mut() {
                    func(b1.borrow_mut(), b2.borrow_mut());
                }
                arr = x;
            }
            None => break,
        }
    }
}
