#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_desktop::{use_window, Config, WindowBuilder};

fn main() {
    // simple_logger::init_with_level(log::Level::Debug).unwrap();
    dioxus_desktop::launch_cfg(
        App,
        Config::default().with_window(WindowBuilder::new().with_resizable(true).with_inner_size(
            dioxus_desktop::wry::application::dpi::LogicalSize::new(400.0, 800.0),
        )),
    );
}

#[component]
fn App(cx: Scope) -> Element {
    cx.render(rsx!("hello world!"))
}
