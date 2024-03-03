#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use theramin::manymouse::ManyMouse;

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
    use_effect(cx, (), |()| {
        tokio::spawn(async move {
            let mut m_mouse = ManyMouse::new();
            println!("{}", m_mouse.driver_name());
            println!("{:?}", m_mouse.device_list());
            loop {
                for ev in m_mouse.poll() {
                    println!("{:?}", ev)
                }
            }
        })
    });
    cx.render(rsx!("hello world!"))
}
