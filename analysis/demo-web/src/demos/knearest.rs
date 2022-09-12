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
    point:Vec2<f32>,
    verts: Vec<axgeom::Rect<f32>>,
}
impl broccoli::queries::knearest::Knearest<BBox<f32, ()>> for MyKnearest {
    fn source(&self)->[&f32;2]{
        [&self.point.x,&self.point.y]
    }
    fn distance_1d(&mut self, start:f32, val: f32) -> f32 {
        let dis = (val - start).abs();
        dis * dis
    }

    fn distance_to_broad(
        &mut self,
        _a: &BBox<f32, ()>,
    ) -> Option<f32> {
        None
    }

    fn distance_to_fine(&mut self,  a: &BBox<f32, ()>) -> f32 {
        self.verts.push(a.rect);
        distance_to_rect(&a.rect, self.point)
    }
}

pub fn make_demo(dim: Rect<f32>, ctx: &CtxWrap) -> impl FnMut(DemoData) {
    let mut bots = support::make_rand_rect(dim, [1.0, 8.0])
        .take(500)
        .map(|rect| bbox(rect, ()))
        .collect::<Vec<_>>();

    let rect_save = {
        let mut verts = vec![];
        for bot in bots.iter() {
            verts.rect(bot.rect.inner_as());
        }
        ctx.buffer_static(&verts)
    };

    let tree_data = broccoli::Tree::new(&mut bots).get_tree_data();

    let mut verts = vec![];
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

        let mut handler = MyKnearest { verts: vec![] ,point:cursor};

        if check_naive {
            let handler2 = MyKnearest { verts: vec![] ,point:cursor};
            Assert::new(&mut bots.clone()).assert_k_nearest_mut( 3, handler2);
        }

        let mut tree = broccoli::Tree::from_tree_data(&mut bots, &tree_data);
        broccoli::assert::assert_tree_invariants(&tree);

        verts.clear();

        ctx.draw_clear([0.13, 0.13, 0.13, 1.0]);

        let mut camera = sys.view(vec2(dim.x.end, dim.y.end), [0.0, 0.0]);

        let mut vv = {
            let (h,k) = tree.find_knearest( 3, handler);
            handler=h;
            drop(handler);

            buffer.update(&verts);

            camera.draw_triangles(&buffer, &[1.0, 1.0, 0.0, 0.3]);

            k
        };

        camera.draw_triangles(&rect_save, &[0.0, 0.1, 0.0, 0.3]);
        for (k, color) in vv.iter().rev().zip(cols.iter()) {
            verts.clear();
            verts.push(cursor.into());
            buffer.update(&verts);
            let radius = k[0].mag.sqrt() * 2.0;
            camera.draw_circles(&buffer, radius, color);

            verts.clear();
            for b in k.iter() {
                verts.rect(b.bot.rect);
            }
            buffer.update(&verts);
            camera.draw_triangles(&buffer, color);
        }

        ctx.flush();
    }
}
