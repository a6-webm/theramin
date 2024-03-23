#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use std::io;
use theramin::{
    input::InputHandler,
    manymouse::{self, Axis, Button, ManyMouse},
    midi::{MidiInitialiser, Pitch, HIGHEST_MIDI_NOTE},
    use_theramin_routine::*,
};

struct Dev {
    name: String,
    selected: bool,
}

impl Dev {
    fn new(name: &str) -> Self {
        Dev {
            name: name.to_string(),
            selected: false,
        }
    }
}

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

    let dev_list = use_ref(cx, || {
        ["mouse_1", "mouse_2", "mouse_3"]
            .iter()
            .map(|s| Dev::new(s))
            .collect::<Vec<Dev>>()
    });

    render! {
        style {
            "html, body {{
                margin: 0;
                padding: 0;
                color: white;
            }}
            "
        },
        div {
            width: "100vw",
            height: "100vh",
            background_color: "#00001a",
            display: "flex",
            flex_direction: "row",
            DevBar { dev_list: dev_list },
            ThereminList { dev_list: dev_list },
        }
    }
}

#[component]
fn DevBar<'a>(cx: Scope, dev_list: &'a UseRef<Vec<Dev>>) -> Element {
    render! {
        div {
            flex: "0 0 12em",
            border: "solid white",
            RefreshButton {},
            DevList { dev_list: dev_list },
        }
    }
}

#[component]
fn RefreshButton(cx: Scope) -> Element {
    render! {
        button {
            "type": "button",
            display: "block",
            margin: "0 auto",
            "Refresh"
        }
    }
}

#[component]
fn DevList<'a>(cx: Scope, dev_list: &'a UseRef<Vec<Dev>>) -> Element {
    dev_list.with(|list| {
        let devs = list.iter().enumerate().map(|(i, d)| (i, &d.name));
        render! {
            div {
                for (i, dev_name) in devs {
                    button {
                        "type": "button",
                        width: "100%",
                        display: "block",
                        onclick: move |_| {
                            dev_list.with_mut(|list| {
                                list[i].selected = !list[i].selected;
                            });
                        },
                        "{dev_name}",
                    }
                }
            }
        }
    })
}

#[component]
fn ThereminList<'a>(cx: Scope, dev_list: &'a UseRef<Vec<Dev>>) -> Element {
    dev_list.with(|list| {
        let enabled_dev_names = list.iter().filter(|d| d.selected).map(|d| &d.name);
        render! {
            div {
                flex: "1 1 100%",
                min_width: "0",
                border: "solid white",
                for dev_name in enabled_dev_names {
                    Theremin {
                        name: "{dev_name}"
                    }
                }
            }
        }
    })
}

#[component]
fn Theremin<'a>(cx: Scope, name: &'a str) -> Element {
    render! {
        div {
            div {
                name
            },
            NoteBar {
                note_width: 4.0,
                note_scroll: 127.5,
            },
        }
    }
}

#[component]
fn NoteBar(cx: Scope, note_width: f32, note_scroll: f32) -> Element {
    let offset = 50.0 - note_scroll * note_width;
    render! {
        div {
            display: "block",
            overflow: "clip",
            white_space: "nowrap",
            div {
                margin_left: "{offset}%",
                display: "inline",
            }
            for note in 0..(HIGHEST_MIDI_NOTE+1) {
                div {
                    width: "{note_width}%",
                    box_sizing: "border-box",
                    border: "solid grey",
                    text_align: "center",
                    display: "inline-block",
                    white_space: "nowrap",
                    note.to_string()
                }
            },
            div {
                text_align: "center",
                "^"
            }
        }
    }
}
