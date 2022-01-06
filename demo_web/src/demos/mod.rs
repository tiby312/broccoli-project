mod intersect_with;
mod knearest;
mod liquid;
mod multirect;
mod nbody;
mod original_order;
mod raycast;
mod raycast_debug;

pub struct DemoData<'a> {
    cursor: Vec2<f32>,
    sys: &'a mut shogo::dots::ShaderSystem,
    ctx: &'a CtxWrap,
    check_naive: bool,
}

pub struct Demo(Box<dyn FnMut(DemoData)>);
impl Demo {
    pub fn new(func: impl FnMut(DemoData) + 'static) -> Self {
        Demo(Box::new(func))
    }
    pub fn step(
        &mut self,
        cursor: Vec2<f32>,
        sys: &mut shogo::dots::ShaderSystem,
        ctx: &CtxWrap,
        check_naive: bool,
    ) {
        self.0(DemoData {
            cursor,
            sys,
            ctx,
            check_naive,
        });
    }
}

use crate::*;
pub struct DemoIter(usize);

impl DemoIter {
    pub fn new() -> DemoIter {
        DemoIter(0)
    }
    pub fn next(&mut self, area: Vec2<u32>, ctx: &shogo::dots::CtxWrap) -> Demo {
        let curr = self.0;
        //let k=ctx.shader_system();

        let area = Rect::new(0.0, area.x as f32, 0.0, area.y as f32);

        let k: Demo = match curr {
            0 => Demo::new(liquid::make_demo(area, ctx)),
            1 => Demo::new(raycast::make_demo(area, ctx)),
            2 => Demo::new(nbody::make_demo(area, ctx)),
            3 => Demo::new(multirect::make_demo(area, ctx)),
            4 => Demo::new(knearest::make_demo(area, ctx)),
            5 => Demo::new(original_order::make_demo(area, ctx)),
            6 => Demo::new(raycast_debug::make_demo(area, ctx)),
            7 => Demo::new(intersect_with::make_demo(area, ctx)),
            _ => unreachable!(),
        };
        self.0 += 1;

        if self.0 == 8 {
            self.0 = 0
        }
        k
    }
}
