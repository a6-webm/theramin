#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_desktop::{Config, LogicalSize, WindowBuilder};

use theramin::{midi::HIGHEST_MIDI_NOTE, use_theramin_routine::*};

fn main() {
    dioxus_desktop::launch_cfg(
        App,
        Config::default()
            .with_window(WindowBuilder::new().with_inner_size(LogicalSize::new(400.0, 800.0))),
    );
}

#[component]
fn App(cx: Scope) -> Element {
    use_theramin_routine(cx);
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
            DevBar {},
            ThereminList {},
        }
    }
}

#[component]
fn DevBar(cx: Scope) -> Element {
    render! {
        div {
            flex: "0 0 12em",
            border: "solid white",
            RefreshButton {},
            DevList {},
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
fn DevList<'a>(cx: Scope) -> Element {
    let device_list = use_shared_state::<Devices>(cx).unwrap();
    device_list.with(|list| {
        render! {
            div {
                for dev in list.iter().cloned() {
                    button {
                        "type": "button",
                        width: "100%",
                        display: "block",
                        onclick: move |_| {
                            device_list.to_owned().with_mut(|list| {
                                list[dev.id].selected = !list[dev.id].selected;
                            });
                        },
                        "{dev.name}",
                    }
                }
            }
        }
    })
}

#[component]
fn ThereminList<'a>(cx: Scope) -> Element {
    let device_list = use_shared_state::<Devices>(cx).unwrap();
    device_list.with(|list| {
        render! {
            div {
                flex: "1 1 100%",
                min_width: "0",
                border: "solid white",
                for dev in list.iter().filter(|d| d.selected) {
                    Theremin {
                        dev: dev.clone()
                    }
                }
            }
        }
    })
}

#[component]
fn Theremin(cx: Scope, dev: Dev) -> Element {
    let mouse_pos = use_mouse_pos(cx, dev.id);
    render! {
        div {
            div {
                "{dev.name}"
            },
            NoteBar {
                note_width: 4.0,
                note_scroll: mouse_pos,
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
