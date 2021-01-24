use crate::support::prelude::*;

pub fn make_demo(dim: Rect<f32>, canvas: &mut SimpleCanvas) -> Demo {
    let bots = support::make_rand_rect(200, dim, [5.0, 20.0], |rect| bbox(rect.inner_as(), ()))
        .into_boxed_slice();

    let mut tree = broccoli::container::TreeOwned::new_par(RayonJoin, bots);

    let mut rects = canvas.rects();
    for bot in tree.as_tree().get_elements().iter() {
        rects.add(bot.rect.inner_as().into());
    }
    let rect_save = rects.save(canvas);

    Demo::new(move |cursor, canvas, check_naive| {
        rect_save
            .uniforms(canvas)
            .with_color([0.0, 1.0, 0.0, 0.2])
            .draw();

        let tree = tree.as_tree_mut();

        let cc: Vec2<i32> = cursor.inner_as();
        let r1 = axgeom::Rect::new(cc.x - 100, cc.x + 100, cc.y - 100, cc.y + 100);
        let r2 = axgeom::Rect::new(100, 400, 100, 400);

        if check_naive {
            use broccoli::query::rect::*;

            assert_for_all_in_rect_mut(tree, &r1);
            assert_for_all_in_rect_mut(tree, &r2);
            assert_for_all_intersect_rect_mut(tree, &r1);
            assert_for_all_intersect_rect_mut(tree, &r2);
            assert_for_all_not_in_rect_mut(tree, &r1);
        }

        //test MultiRect
        {
            let mut rects = tree.multi_rect();

            let mut to_draw = Vec::new();
            let _ = rects.for_all_in_rect_mut(r1, |a| to_draw.push(a));

            let res = rects.for_all_in_rect_mut(r2, |a| {
                to_draw.push(a);
            });

            match res {
                Ok(()) => {
                    canvas
                        .rects()
                        .add(r1.inner_as().into())
                        .add(r2.inner_as().into())
                        .send_and_uniforms(canvas)
                        .with_color([0.0, 0.0, 0.0, 0.5])
                        .draw();

                    let mut rects = canvas.rects();
                    for r in to_draw.iter() {
                        rects.add(r.rect.inner_as().into());
                    }
                    rects
                        .send_and_uniforms(canvas)
                        .with_color([0.0, 0.0, 0.0, 0.2])
                        .draw();
                }
                Err(_) => {
                    canvas
                        .rects()
                        .add(r1.inner_as().into())
                        .add(r2.inner_as().into())
                        .send_and_uniforms(canvas)
                        .with_color([1.0, 0.0, 0.0, 0.5])
                        .draw();
                }
            }
        }

        //test for_all_intersect_rect
        let mut rects = canvas.rects();
        tree.for_all_intersect_rect(&r1, |a| {
            rects.add(a.rect.inner_as().into());
        });
        rects
            .send_and_uniforms(canvas)
            .with_color([0.0, 0.0, 1.0, 0.2])
            .draw();

        canvas
            .rects()
            .add(r1.inner_as().into())
            .send_and_uniforms(canvas)
            .with_color([1.0, 0.0, 0.0, 0.2])
            .draw();

        //test for_all_not_in_rect_mut
        let mut rects = canvas.rects();
        tree.for_all_not_in_rect_mut(&r1, |b| {
            rects.add(b.rect.inner_as().into());
        });
        rects
            .send_and_uniforms(canvas)
            .with_color([1.0, 0.0, 0.0, 0.5])
            .draw();
    })
}
