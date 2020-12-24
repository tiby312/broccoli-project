use super::inner_prelude::*;
//this can have some false positives.
//but it will still prune a lot of bots.
#[inline(always)]
pub fn get_section<'a, I: Aabb, A: Axis>(axis: A, arr: &'a [I], range: Range<I::Num>) -> &'a [I] {
    if arr.is_empty() {
        return arr;
    }

    let ll = arr.len();
    let mut start = None;
    for (e, i) in arr.as_ref().iter().enumerate() {
        let rr = i.get().get_range(axis);
        if e == ll - 1 || rr.end >= range.start {
            start = Some(e);
            break;
        }
    }

    let start = start.unwrap();

    let mut end = arr.as_ref().len();
    for (e, i) in arr[start..].iter().enumerate() {
        let rr = i.get().get_range(axis);
        if rr.start > range.end {
            end = start + e;
            break;
        }
    }

    &arr[start..end]
}

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
        PMut::new(&mut aabbs),
        axgeom::Range::new(5, 10),
    );
    let k: &[axgeom::Rect<isize>] = &k;
    assert_eq!(k.len(), 3);
}

//this can have some false positives.
//but it will still prune a lot of bots.
#[inline(always)]
pub fn get_section_mut<'a, I: Aabb, A: Axis>(
    axis: A,
    mut arr: PMut<'a, [I]>,
    range: Range<I::Num>,
) -> PMut<'a, [I]> {
    if arr.is_empty() {
        return arr;
    }

    let ll = arr.len();
    let mut start = None;
    for (e, i) in arr.as_ref().iter().enumerate() {
        let rr = i.get().get_range(axis);
        if e == ll - 1 || rr.end >= range.start {
            start = Some(e);
            break;
        }
    }

    let start = start.unwrap();

    let mut end = arr.as_ref().len();
    for (e, i) in arr.borrow_mut().truncate_from(start..).iter().enumerate() {
        let rr = i.get().get_range(axis);
        if rr.start > range.end {
            end = start + e;
            break;
        }
    }

    arr.truncate(start..end)
}

pub fn for_every_pair<T: Aabb>(mut arr: PMut<[T]>, mut func: impl FnMut(PMut<T>, PMut<T>)) {
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
