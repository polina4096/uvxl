#![feature(const_trait_impl)]
#![feature(duration_consts_float)]

#![allow(dead_code)]
#![allow(unused_variables)]

#![allow(clippy::needless_return)]
#![allow(clippy::needless_pass_by_ref_mut)]
#![allow(clippy::single_match)]
#![allow(clippy::module_inception)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::type_complexity)]

extern crate core;

use cfg_if::cfg_if;

cfg_if! {
  if #[cfg(feature = "client")] {
    pub mod app;
    pub mod input;
    pub mod graphics;
    pub mod network;
  }
}

pub mod game;
pub mod util;

#[cfg(feature = "server")]
pub mod server;

cfg_if! {
  if #[cfg(feature = "client")] {
    use app::UVxlEvent;
    use winit::dpi::LogicalSize;
    use winit::event_loop::{EventLoop, EventLoopBuilder};
    use winit::window::{Window, WindowBuilder};

    use crate::app::App;

    // WASM pointer locking hacks
    #[cfg(target_arch="wasm32")]
    use wasm_bindgen::prelude::*;

    // WASM entrypoint
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(target_arch="wasm32", wasm_bindgen)]
    pub async fn run_wasm(container_selector: &str) {
      let (window, event_loop) = run().await.unwrap();
      let web_window = web_sys::window().expect("Failed to get web window object");

      // handle browser resize events
      let window_ptr = std::ptr::addr_of!(window);
      let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
        let window = unsafe { &*window_ptr };
        let web_window = web_sys::window().expect("Failed to get web window object");
        let size = LogicalSize::new(
          web_window.inner_width().unwrap().as_f64().unwrap(),
          web_window.inner_height().unwrap().as_f64().unwrap(),
        );

        window.set_inner_size(size);
      }) as Box<dyn FnMut(_)>);

      web_window.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref()).unwrap();

      // append canvas to the window
      use winit::platform::web::WindowExtWebSys;

      web_window.document()
        .and_then(|doc| doc.query_selector(container_selector).ok()?)
        .and_then(|dst| dst.append_child(&web_sys::Element::from(window.canvas())).ok())
        .expect("Failed to append canvas to document body");

      let _ = web_window.set_timeout_with_callback(closure.as_ref().unchecked_ref())
        .map_err(|err| log::error!("Failed to set initial size for canvas: {:?}", err));

      App::run(window, event_loop).await;

      closure.forget(); // Here we leak memory, but it's ok since this closure should have 'static anyways
    }

    // default entrypoint
    pub async fn run() -> Option<(Window, EventLoop<UVxlEvent>)> {
      cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
          console_error_panic_hook::set_once();
          console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
        } else { pretty_env_logger::init(); }
      }

      let event_loop = EventLoopBuilder::with_user_event()
        .build();

      let window = WindowBuilder::new()
        .with_title("uvxl")
        .with_inner_size(LogicalSize::new(1000, 625))
        .build(&event_loop)
        .unwrap();

      cfg_if! { if #[cfg(target_arch = "wasm32")]
        { return Some((window, event_loop)); } else {
          App::run(window, event_loop).await;
          return None;
        }
      }
    }
  }
}