use broccoli::assert::Assert;

use super::*;

fn distance_to_line(point: Vec2<f32>, axis: impl Axis, val: f32) -> f32 {
    let dis = (val - *point.get_axis(axis)).abs();
    dis * dis
}
fn distance_to_rect(rect: &Rect<f32>, point: Vec2<f32>) -> f32 {
    let dis = rect.distance_squared_to_point(point);
    let dis = match dis {
        Some(dis) => dis,
        None => {
            //If a point is insert a rect, the distance to it is zero.
            //So if multiple points are inside of a rect, its not clear the order in which
            //they should be returned.
            //So in the case that a point is in the rect, we establish our own ordering,
            //by falling back on the distance between the center of a rect and the point.
            //Since the distance between a rect and a point that is outside of the rect is
            //guaranteed to be positive, we have all the negative numbers in which to
            //apply our custom ordering for bots that are inside of the rect.

            //The main reason that we are doing this is so that there arn't
            //multiple solutions to the k_nearest problem so that we can easily
            //verify the solution against the naive implementation.

            //If you don't care about a single solution existing, you can simply return zero
            //for the cases that the point is inside of the rect.

            0.0
        }
    };
    dis
}

struct MyKnearest {
    verts: Vec<axgeom::Rect<f32>>,
}
impl broccoli::queries::knearest::Knearest<BBox<f32, ()>> for MyKnearest {
    fn distance_to_aaline<A: Axis>(&mut self, point: Vec2<f32>, axis: A, val: f32) -> f32 {
        distance_to_line(point, axis, val)
    }

    fn distance_to_broad(
        &mut self,
        _point: Vec2<f32>,
        _a: AabbPin<&mut BBox<f32, ()>>,
    ) -> Option<f32> {
        None
    }

    fn distance_to_fine(&mut self, point: Vec2<f32>, a: AabbPin<&mut BBox<f32, ()>>) -> f32 {
        self.verts.push(a.rect);
        distance_to_rect(&a.rect, point)
    }
}

pub fn make_demo(dim: Rect<f32>, ctx: &CtxWrap) -> impl FnMut(DemoData) {
    let mut bots = support::make_rand_rect(dim, [1.0, 8.0])
        .take(500)
        .map(|rect| bbox(rect, ()))
        .collect::<Vec<_>>();

    let mut verts = vec![];

    let mut s = simple2d::shapes(&mut verts);
    for bot in bots.iter() {
        s.rect(bot.rect.inner_as());
    }

    let rect_save = ctx.buffer_static_clear(&mut verts);

    let tree_data = broccoli::Tree::new(&mut bots).get_tree_data();

    let mut buffer = ctx.buffer_dynamic();

    move |data| {
        let DemoData {
            cursor,
            sys,
            ctx,
            check_naive,
        } = data;

        let cols = [
            [1.0, 0.0, 0.0, 0.3], //red closest
            [0.0, 1.0, 0.0, 0.3], //green second closest
            [0.0, 0.0, 1.0, 0.3], //blue third closets
        ];

        let mut handler = MyKnearest { verts: vec![] };

        if check_naive {
            Assert::new(&mut bots.clone()).assert_k_nearest_mut(cursor, 3, &mut handler);
            handler.verts.clear();
        }

        let mut tree = broccoli::Tree::from_tree_data(&mut bots, &tree_data);
        broccoli::assert::assert_tree_invariants(&tree);

        ctx.draw_clear([0.13, 0.13, 0.13, 1.0]);

        let mut camera = sys.view(vec2(dim.x.end, dim.y.end), [0.0, 0.0]);

        let mut vv = {
            let k = tree.find_knearest(cursor, 3, &mut handler);
            drop(handler);

            buffer.update_clear(&mut verts);

            camera.draw_triangles(&buffer, &[1.0, 1.0, 0.0, 0.3]);

            k
        };

        camera.draw_triangles(&rect_save, &[0.0, 0.1, 0.0, 0.3]);

        for (k, color) in vv.iter().rev().zip(cols.iter()) {
            verts.push(cursor.into());
            buffer.update_clear(&mut verts);
            let radius = k[0].mag.sqrt() * 2.0;
            camera.draw_circles(&buffer, radius, color);

            let mut s = simple2d::shapes(&mut verts);
            for b in k.iter() {
                s.rect(b.bot.rect);
            }
            buffer.update_clear(&mut verts);
            camera.draw_triangles(&buffer, color);
        }

        ctx.flush();
    }
}
