#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use std::io;
use theramin::{
    input::InputHandler,
    manymouse::{self, Axis, Button, ManyMouse},
    midi::MidiInitialiser,
    use_theramin_routine::*,
};

fn main() {
    // simple_logger::init_with_level(log::Level::Debug).unwrap();
    dioxus_desktop::launch_cfg(
        App,
        Config::default().with_window(
            WindowBuilder::new(), // .with_inner_size(dioxus_desktop::wry::application::dpi::LogicalSize::new(400.0, 800.0),)
        ),
    );
}

#[component]
fn App(cx: Scope) -> Element {
    // let rx = use_theramin_routine(cx, 0u32, |tx| async move {
    //     let mut m_mouse = ManyMouse::new();
    //     println!("{}", m_mouse.driver_name());
    //     println!("{:?}", m_mouse.device_list());

    //     let mut buffer = String::new();
    //     let stdin = io::stdin();
    //     print!("Which dev bruh: ");
    //     stdin.read_line(&mut buffer).unwrap();
    //     let dev_idx: u32 = buffer.trim().parse().unwrap();

    //     let mut midi_h = MidiInitialiser::new().virtual_port("port_1");

    //     let mut input_h = InputHandler::new(50);

    //     loop {
    //         for ev in m_mouse.poll() {
    //             println!("{:?}", ev);
    //             if ev.device != dev_idx {
    //                 continue;
    //             }
    //             match ev.ev_type {
    //                 manymouse::EventType::Relmotion if ev.item == Button::LMB as u32 => {
    //                     let pitch = input_h.handle_rel_move(ev.value);
    //                     if input_h.playing {
    //                         midi_h.play(pitch);
    //                     }
    //                     tx.send(input_h.pos()).unwrap();
    //                 }
    //                 manymouse::EventType::Button if ev.item == Axis::X as u32 => {
    //                     input_h.playing = ev.value == 1;
    //                     if input_h.playing {
    //                         midi_h.play(input_h.pitch_from_pos());
    //                     } else {
    //                         midi_h.release();
    //                     }
    //                 }
    //                 _ => (),
    //             }
    //         }
    //     }
    // });

    // let pos = rx.read().unwrap().to_owned();

    render! {
        style {
            "html, body {{
                margin: 0;
                padding: 0;
            }}
            * {{
                box-sizing: border-box;
            }}
            "
        },
        div {
            width: "100vw",
            height: "100vh",
            border: "dashed red",
            background_color: "lightskyblue",
            display: "flex",
            flex_direction: "row",
            DevBar {},
            ThereminList {},
        }
    }
}

#[component]
fn DevBar(cx: Scope) -> Element {
    render! {
        div {
            flex: "0 1 12em",
            border: "dashed red",
            RefreshButton {},
            DevList {},
        }
    }
}

#[component]
fn RefreshButton(cx: Scope) -> Element {
    render! {
        "Refresh"
    }
}

#[component]
fn DevList(cx: Scope) -> Element {
    let uh = vec!["mouse_1", "mouse_2", "mouse_3"];
    render! {
        div {
            for dev in uh {
                div {
                    dev
                }
            }
        }
    }
}

#[component]
fn ThereminList(cx: Scope) -> Element {
    let uh = vec!["mouse_1", "mouse_2", "mouse_3"];
    render! {
        div {
            flex: "1 1 auto",
            border: "dashed red",
            for dev in uh {
                Theremin {
                    name: dev
                }
            }
        }
    }
}

#[component]
fn Theremin<'a>(cx: Scope, name: &'a str) -> Element {
    render! {
        div {
            name
        }
    }
}
