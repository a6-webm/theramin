#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_desktop::{Config, LogicalSize, WindowBuilder};

use theramin::{midi::HIGHEST_MIDI_NOTE, use_theramin_routine::*};

fn main() {
    dioxus_desktop::launch::launch(
        App,
        vec![],
        Config::default()
            .with_window(WindowBuilder::new().with_inner_size(LogicalSize::new(400.0, 800.0))),
    );
}

#[component]
fn App() -> Element {
    use_theramin_routine();
    rsx! {
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
fn DevBar() -> Element {
    rsx! {
        div {
            flex: "0 0 12em",
            border: "solid white",
            RefreshButton {},
            DevList {},
        }
    }
}

#[component]
fn RefreshButton() -> Element {
    rsx! {
        button {
            "type": "button",
            display: "block",
            margin: "0 auto",
            "Refresh"
        }
    }
}

#[component]
fn DevList() -> Element {
    let devices: Signal<Devices> = use_context();
    let theramin_msg_tx: Signal<TheraminMsgTx> = use_context();
    devices.with(|devices| {
        let buttons = devices.iter().cloned().map(|dev| {
            let text = format!("{}{}", dev.name, if dev.selected { "x " } else { "" });
            rsx! {
                button {
                    "type": "button",
                    width: "100%",
                    display: "block",
                    onclick: move |_| {
                        theramin_msg_tx.read().send(Msg::ClickDev(dev.id));
                    },
                    "{text}",
                }
            }
        });
        rsx! {
            div {
                {buttons}
            }
        }
    })
}

#[component]
fn ThereminList() -> Element {
    println!("rendered ThereminList");
    let devices: Signal<Devices> = use_context();
    let theremin_positions = use_theremin_positions();
    rsx! {
        div {
            flex: "1 1 100%",
            min_width: "0",
            border: "solid white",
            for dev in devices.iter().filter(|d| d.selected) {
                div {
                    div {
                        "{dev.name}"
                    },
                    NoteBar {
                        note_width: 4.0, // TODO be able to change
                        note_scroll: theremin_positions.read()[dev.id],
                    },
                }
            }
        }
    }
}

#[component]
fn NoteBar(note_width: f32, note_scroll: f32) -> Element {
    println!("rendered NoteBar");
    let offset = 50.0 - note_scroll * note_width;
    rsx! {
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
                    "{note}"
                }
            },
            div {
                text_align: "center",
                "^"
            }
        }
    }
}
