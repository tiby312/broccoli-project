use gloo::console::log;
use serde::{Deserialize, Serialize};
use shogo::utils;
use wasm_bindgen::{prelude::*, JsCast};

const COLORS: &[[f32; 4]] = &[
    [1.0, 0.0, 0.0, 0.5],
    [0.0, 1.0, 0.0, 0.5],
    [0.0, 0.0, 1.0, 0.5],
];



use axgeom::*;
use web_sys::WebGl2RenderingContext;

pub struct Demo(Box<dyn FnMut(Vec2<f32>, &mut shogo::dots::ShaderSystem, &WebGl2RenderingContext, bool)>);
impl Demo {
    pub fn new(func: impl FnMut(Vec2<f32>, &mut shogo::dots::ShaderSystem, &WebGl2RenderingContext, bool) + 'static) -> Self {
        Demo(Box::new(func))
    }
    pub fn step(&mut self, point: Vec2<f32>, sys:&mut shogo::dots::ShaderSystem, ctx:&WebGl2RenderingContext, check_naive: bool) {
        self.0(point, sys,ctx, check_naive);
    }
}


mod demos;
mod support;
pub use crate::support::prelude::*;



///Common data sent from the main thread to the worker.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MEvent {
    CanvasMouseMove { x: f32, y: f32 },
    ButtonClick,
    ShutdownClick,
}

#[wasm_bindgen]
pub async fn main_entry() {
    use futures::StreamExt;

    log!("demo start");

    let (canvas, button, shutdown_button) = (
        utils::get_by_id_canvas("mycanvas"),
        utils::get_by_id_elem("mybutton"),
        utils::get_by_id_elem("shutdownbutton"),
    );

    let offscreen = canvas.transfer_control_to_offscreen().unwrap_throw();

    let (mut worker, mut response) = shogo::EngineMain::new(offscreen).await;

    let _handler = worker.register_event(&canvas, "mousemove", |e| {
        let [x, y] = convert_coord(e.elem, e.event);
        MEvent::CanvasMouseMove { x, y }
    });

    let _handler = worker.register_event(&button, "click", |_| MEvent::ButtonClick);

    let _handler = worker.register_event(&shutdown_button, "click", |_| MEvent::ShutdownClick);

    let _: () = response.next().await.unwrap_throw();
    log!("main thread is closing");
}

#[wasm_bindgen]
pub async fn worker_entry() {
    use shogo::dots::{CtxExt, Shapes};

    let (mut w, ss) = shogo::EngineWorker::new().await;
    let mut frame_timer = shogo::FrameTimer::new(30, ss);

    let canvas = w.canvas();

    let ctx = utils::get_context_webgl2_offscreen(&canvas);

    let mut mouse_pos = [0.0f32; 2];

    let mut color_iter = COLORS.iter().cycle().peekable();

    let (mut draw_sys, mut buffer, walls) = (
        ctx.shader_system(),
        ctx.buffer_dynamic(),
        ctx.buffer_static(vec![].rect(30.0, [40.0, 40.0], [800.0 - 80.0, 600.0 - 80.0])),
    );

    let mut verts = vec![];
    'outer: loop {
        for e in frame_timer.next().await {
            match e {
                MEvent::CanvasMouseMove { x, y } => mouse_pos = [*x, *y],
                MEvent::ButtonClick => {
                    let _ = color_iter.next();
                }
                MEvent::ShutdownClick => break 'outer,
            }
        }

        let radius = 8.0;
        let game_dim = [canvas.width() as f32, canvas.height() as f32];

        verts.clear();
        verts.line(radius, mouse_pos, [0.0, 0.0]);
        verts.line(radius, mouse_pos, game_dim);
        verts.line(radius, mouse_pos, [0.0, game_dim[1]]);
        verts.line(radius, mouse_pos, [game_dim[0], 0.0]);
        buffer.update(&verts);


        ctx.clear_color(0.13, 0.13, 0.13, 1.0);
        ctx.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);
        


        draw_sys.draw_circles(
            &buffer,
            game_dim,
            color_iter.peek().unwrap_throw(),
            [0.0, 0.0],
            radius,
        );
        draw_sys.draw_squares(&walls, game_dim, &[1.0, 1.0, 1.0, 0.2], [0.0, 0.0], radius);
    }

    w.post_message(());

    log!("worker thread closing");
}

fn convert_coord(canvas: &web_sys::HtmlElement, event: &web_sys::Event) -> [f32; 2] {
    let e = event
        .dyn_ref::<web_sys::MouseEvent>()
        .unwrap_throw()
        .clone();

    let [x, y] = [e.client_x() as f32, e.client_y() as f32];
    let bb = canvas.get_bounding_client_rect();
    let tl = bb.x() as f32;
    let tr = bb.y() as f32;
    [x - tl, y - tr]
}
