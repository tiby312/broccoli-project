use crate::pmut::PMut;
use crate::Aabb;

pub(crate) fn for_every_pair<T: Aabb>(mut arr: PMut<[T]>, mut func: impl FnMut(PMut<T>, PMut<T>)) {
    loop {
        let temp = arr;
        match temp.split_first_mut() {
            Some((mut b1, mut x)) => {
                for mut b2 in x.as_mut().iter_mut() {
                    func(b1.as_mut(), b2.as_mut());
                }
                arr = x;
            }
            None => break,
        }
    }
}
