use crate::support::prelude::*;

use axgeom::Ray;

pub fn make_demo(dim: Rect<f32>, canvas: &mut SimpleCanvas) -> Demo {
    let walls = support::make_rand_rect(5000, dim, [1.0, 4.0], |a| bbox(a, ())).into_boxed_slice();

    let mut counter: f32 = 0.0;
    let mut tree = broccoli::container::TreeOwned::new_par(walls);

    let mut rects = canvas.rects();
    for bot in tree.as_tree().get_bbox_elements().iter() {
        rects.add(bot.rect.into());
    }
    let rect_save = rects.save(canvas);

    Demo::new(move |cursor, canvas, check_naive| {
        let tree = tree.as_tree_mut();

        let ray: Ray<f32> = {
            counter += 0.004;
            let point: Vec2<f32> = cursor;
            let dir = vec2(counter.cos() * 10.0, counter.sin() * 10.0);

            Ray { point, dir }
        };

        //Draw the walls
        rect_save
            .uniforms(canvas)
            .with_color([0.0, 0.0, 0.0, 0.3])
            .draw();

        struct Foo {
            rects: egaku2d::shapes::RectSession,
        }
        impl broccoli::query::RayCast for Foo {
            type T = BBox<f32, ()>;
            type N = f32;

            fn compute_distance_to_aaline<A: Axis>(
                &mut self,
                ray: &Ray<Self::N>,
                axis: A,
                val: Self::N,
            ) -> axgeom::CastResult<Self::N> {
                ray.cast_to_aaline(axis, val)
            }

            ///Returns true if the ray intersects with this rectangle.
            ///This function allows as to prune which nodes to visit.
            fn compute_distance_to_rect(
                &mut self,
                ray: &Ray<Self::N>,
                a: &Rect<Self::N>,
            ) -> axgeom::CastResult<Self::N> {
                self.rects.add(a.into());
                ray.cast_to_rect(a)
            }

            ///The expensive collision detection
            ///This is where the user can do expensive collision detection on the shape
            ///contains within it's bounding box.
            ///Its default implementation just calls compute_distance_to_rect()
            fn compute_distance_to_bot(
                &mut self,
                ray: &Ray<Self::N>,
                a: &Self::T,
            ) -> axgeom::CastResult<Self::N> {
                ray.cast_to_rect(&a.rect)
            }
        }

        let rects = canvas.rects();
        let mut foo = Foo { rects };

        if check_naive {
            tree.assert_raycast_mut(ray, &mut foo);
        }

        let test = {
            let test = tree.raycast_mut(ray, &mut foo);
            foo.rects
                .send_and_uniforms(canvas)
                .with_color([4.0, 0.0, 0.0, 0.4])
                .draw();
            test
        };

        let dis = match test {
            axgeom::CastResult::Hit((_, dis)) => dis,
            axgeom::CastResult::NoHit => 800.0,
        };

        let end = ray.point_at_tval(dis);

        canvas
            .lines(2.0)
            .add(ray.point.into(), end.into())
            .send_and_uniforms(canvas)
            .with_color([1., 1., 1., 0.2])
            .draw();
    })
}
