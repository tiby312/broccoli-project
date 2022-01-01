use gloo::console::log;
use serde::{Deserialize, Serialize};
use shogo::utils;
use wasm_bindgen::{prelude::*, JsCast};

use axgeom::*;
use web_sys::WebGl2RenderingContext;

mod demos;
mod support;
pub use crate::support::prelude::*;

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

    use shogo::dots::{CtxExt, Shapes};

    let (mut w, ss) = shogo::EngineWorker::new().await;
    let mut frame_timer = shogo::FrameTimer::new(60, ss);

    let canvas = w.canvas();

    let ctx = utils::get_context_webgl2_offscreen(&canvas);

    let mut mouse_pos = [0.0f32; 2];

    let mut sys = ctx.shader_system();

    let mut demo_iter = demos::DemoIter::new();
    let mut curr = demo_iter.next(area, &ctx);

    let check_naive = false;

    'outer: loop {
        for e in frame_timer.next().await {
            match e {
                MEvent::CanvasMouseMove { x, y } => mouse_pos = [*x, *y],
                MEvent::NextButtonClick => {
                    curr=demo_iter.next(area,&ctx)
                }
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
