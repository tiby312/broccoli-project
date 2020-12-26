use crate::support::prelude::*;



fn distance_to_line(point:Vec2<f32>,axis:impl Axis,val:f32)->f32{
    let dis=(val-*point.get_axis(axis)).abs();
    dis*dis
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
            //guarenteeded to be positive, we have all the negative numbers in which to
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

pub fn make_demo(dim: Rect<f32>, canvas: &mut SimpleCanvas) -> Demo {
    let bots =
        support::make_rand_rect(2000, dim, [1.0, 4.0], |a| bbox(a,())).into_boxed_slice();


    let mut rects = canvas.rects();
    for bot in bots.iter() {
        rects.add(bot.rect.into());
    }
    let rect_save = rects.save(canvas);

    let mut tree = broccoli::container::TreeOwned::new(bots);

    Demo::new(move |cursor, canvas, check_naive| {
        
        
        
        let cols = [
            [1.0, 0.0, 0.0, 0.6], //red closest
            [0.0, 1.0, 0.0, 0.6], //green second closest
            [0.0, 0.0, 1.0, 0.6], //blue third closets
        ];

        
        let mut rects=canvas.rects();
        let mut knearest_stuff=broccoli::query::KnearestClosure::new(
            tree.as_tree(),
            &mut rects,
            |rects,point,rect|{
                rects.add(rect.into());
                distance_to_rect(rect,point)
            },
            |_,point,bot|distance_to_rect(&bot.rect,point),
            |_,point,val|distance_to_line(point,axgeom::XAXIS,val),
            |_,point,val|distance_to_line(point,axgeom::YAXIS,val),
        );

        let tree=tree.as_tree_mut();
        if check_naive {
            
            tree.assert_k_nearest_mut(
                cursor,
                3,
                &mut knearest_stuff
            );
        }
        
          
        let mut vv = {
           
            let k = tree.k_nearest_mut(cursor,3,&mut knearest_stuff);
            rects
                .send_and_uniforms(canvas)
                .with_color([1.0, 1.0, 0.0, 0.3])
                .draw();
            k
        };

        rect_save
            .uniforms(canvas)
            .with_color([0.0, 0.0, 0.0, 0.3])
            .draw();

        for (k, color) in vv.iter().rev().zip(cols.iter()) {
            canvas
                .circles()
                .add(cursor.into())
                .send_and_uniforms(canvas, k[0].mag.sqrt() * 2.0)
                .with_color(*color)
                .draw();

            let mut rects = canvas.rects();
            for b in k.iter() {
                rects.add(b.bot.rect.into());
            }
            rects.send_and_uniforms(canvas).with_color(*color).draw();
        }
    })
}
