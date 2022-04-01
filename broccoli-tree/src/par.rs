use super::*;

pub fn recurse_par<'a, T: Aabb, S: Sorter>(
    vistr: TreeBister<'a, T, S>,
    height_seq_fallback: usize,
    buffer: &mut Vec<Node<'a, T>>,
) where
    T: Send,
    T::Num: Send,
{
    if vistr.get_height() <= height_seq_fallback {
        vistr.recurse_seq(buffer);
    } else {
        let Res { node, rest } = vistr.build_and_next();

        if let Some([left, right]) = rest {
            let (_, mut a) = rayon_core::join(
                || {
                    buffer.push(node.finish());
                    recurse_par(left, height_seq_fallback, buffer);
                },
                || {
                    let mut f = vec![];
                    recurse_par(right, height_seq_fallback, &mut f);
                    f
                },
            );

            buffer.append(&mut a);
        }
    }
}
