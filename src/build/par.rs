use super::*;

pub fn recurse_par<'a, T: Aabb, S: Sorter>(
    vistr: TreeBister<'a, T, S>,
    height_seq_fallback: usize,
    foo: &mut Vec<Node<'a, T>>,
) where
    T: Send + Sync,
    T::Num: Send + Sync,
{
    //TODO is the height of leafs zero???
    if vistr.get_height() <= height_seq_fallback {
        vistr.recurse_seq(foo);
    } else {
        let (n, rest) = vistr.build_and_next();

        if let Some([left, right]) = rest {
            let (_, mut a) = rayon_core::join(
                || {
                    foo.push(n.finish());
                    recurse_par(left, height_seq_fallback, foo);
                },
                || {
                    let mut f = vec![];
                    recurse_par(right, height_seq_fallback, &mut f);
                    f
                },
            );

            foo.append(&mut a);
        }
    }
}
