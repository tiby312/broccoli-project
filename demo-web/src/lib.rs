use gloo::console::log;
use serde::{Deserialize, Serialize};
use shogo::utils;
use wasm_bindgen::{prelude::*, JsCast};

mod demos;
mod support;

pub use crate::dists::*;
pub use broccoli::axgeom;
pub use broccoli::axgeom::*;
pub use broccoli::compt;
use broccoli::tree::aabb_pin::AabbPin;
pub use broccoli::tree::bbox;
pub use broccoli::tree::node::*;
pub use broccoli::tree::*;
//pub use broccoli::rayon;

pub use broccoli::prelude::*;

pub use crate::demos::Demo;
pub use crate::demos::DemoData;
pub use dists::uniform_rand::UniformRandGen;
pub use duckduckgeo::array2_inner_into;
pub use duckduckgeo::*;
pub use shogo::simple2d::CtxWrap;
pub use shogo::simple2d::Shapes;

///Common data sent from the main thread to the worker.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MEvent {
    CanvasMouseMove { x: f32, y: f32 },
    NextButtonClick,
    ShutdownClick,
}

#[wasm_bindgen]
pub async fn main_entry() {
    use futures::StreamExt;

    log!("demo start");

    let (canvas, next_button, shutdown_button) = (
        utils::get_by_id_canvas("mycanvas"),
        utils::get_by_id_elem("nextbutton"),
        utils::get_by_id_elem("shutdownbutton"),
    );

    let offscreen = canvas.transfer_control_to_offscreen().unwrap_throw();

    let (mut worker, mut response) = shogo::EngineMain::new(offscreen).await;

    let _handler = worker.register_event(&canvas, "mousemove", |e| {
        let [x, y] = convert_coord(e.elem, e.event);
        MEvent::CanvasMouseMove { x, y }
    });

    let _handler = worker.register_event(&next_button, "click", |_| MEvent::NextButtonClick);

    let _handler = worker.register_event(&shutdown_button, "click", |_| MEvent::ShutdownClick);

    let _: () = response.next().await.unwrap_throw();
    log!("main thread is closing");
}

#[wasm_bindgen]
pub async fn worker_entry() {
    let area = vec2(800, 600);

    let (mut w, ss) = shogo::EngineWorker::new().await;
    let mut frame_timer = shogo::FrameTimer::new(60, ss);

    let canvas = w.canvas();

    let ctx = shogo::simple2d::CtxWrap::new(&utils::get_context_webgl2_offscreen(&canvas));

    let mut mouse_pos = [0.0f32; 2];

    let mut sys = ctx.shader_system();
    ctx.setup_alpha();
    let mut demo_iter = demos::DemoIter::new();
    let mut curr = demo_iter.next(area, &ctx);

    let check_naive = false;

    'outer: loop {
        for e in frame_timer.next().await {
            match e {
                MEvent::CanvasMouseMove { x, y } => mouse_pos = [*x, *y],
                MEvent::NextButtonClick => curr = demo_iter.next(area, &ctx),
                MEvent::ShutdownClick => break 'outer,
            }
        }

        curr.step(
            Vec2::from(mouse_pos).inner_try_into().unwrap(),
            &mut sys,
            &ctx,
            check_naive,
        );
    }

    w.post_message(());

    log!("worker thread closing");
}

//https://stackoverflow.com/questions/17130395/real-mouse-position-in-canvas
fn convert_coord(canvas: &web_sys::HtmlElement, event: &web_sys::Event) -> [f32; 2] {
    let rect = canvas.get_bounding_client_rect();

    let canvas_width: f64 = canvas
        .get_attribute("width")
        .unwrap_throw()
        .parse()
        .unwrap_throw();
    let canvas_height: f64 = canvas
        .get_attribute("height")
        .unwrap_throw()
        .parse()
        .unwrap_throw();

    let scalex = canvas_width / rect.width();
    let scaley = canvas_height / rect.height();

    let e = event
        .dyn_ref::<web_sys::MouseEvent>()
        .unwrap_throw()
        .clone();

    let [x, y] = [e.client_x() as f64, e.client_y() as f64];

    let [x, y] = [(x - rect.left()) * scalex, (y - rect.top()) * scaley];
    [x as f32, y as f32]
}
