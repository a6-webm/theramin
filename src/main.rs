#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_desktop::{
    tao::keyboard::KeyCode, use_window, use_wry_event_handler, Config, LogicalSize, WindowBuilder,
};

use theramin::{midi::HIGHEST_MIDI_NOTE, use_theramin_routine::*};

fn main() {
    dioxus_desktop::launch::launch(
        App,
        vec![],
        Config::default()
            .with_window(WindowBuilder::new().with_inner_size(LogicalSize::new(400.0, 800.0)))
            .with_menu(None)
            .with_disable_context_menu(true),
    );
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum CursorState {
    NoGrab,
    Grab,
    PendingUnGrab,
}

impl CursorState {
    fn is_grabbed(self) -> bool {
        self != CursorState::NoGrab
    }
}

#[component]
fn App() -> Element {
    use_theramin_routine();
    let cursor_state = use_context_provider(|| Signal::new(CursorState::NoGrab));
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
            if cursor_state.read().is_grabbed() {
                MouseHider {}
            },
            DevBar {},
            ThereminList {},
        }
    }
}

#[component]
fn MouseHider() -> Element {
    let mut cursor_state: Signal<CursorState> = use_context();
    let window = use_window();
    use_effect(move || {
        window.set_cursor_visible(false);
        window.set_cursor_grab(true).unwrap();
    });
    use dioxus_desktop::tao::event::Event::WindowEvent;
    use_wry_event_handler(move |ev, _| {
        if let WindowEvent { event, .. } = ev {
            use dioxus_desktop::tao::event::WindowEvent;
            if let WindowEvent::KeyboardInput { event, .. } = event {
                if event.physical_key == KeyCode::Escape {
                    *cursor_state.write() = CursorState::PendingUnGrab;
                }
            }
        }
    });
    let window = use_window();
    if *cursor_state.read() == CursorState::PendingUnGrab {
        window.set_cursor_visible(true);
        window.set_cursor_grab(false).unwrap();
        *cursor_state.write() = CursorState::NoGrab;
    }
    rsx! {
        div {
            position: "absolute",
            padding: "50vh 50vw",
            top: "0",
            left: "0",
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
    let theramin_msg_tx: Signal<TheraminMsgTx> = use_context();
    rsx! {
        button {
            "type": "button",
            display: "block",
            margin: "0 auto",
            onclick: move |_| {
                theramin_msg_tx.read().send(Msg::FindNewDevices);
            },
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
    let devices: Signal<Devices> = use_context();
    let theremin_positions = use_theremin_positions();
    let mut cursor_state: Signal<CursorState> = use_context();
    rsx! {
        div {
            flex: "1 1 100%",
            min_width: "0",
            border: "solid white",
            button {
                "type": "button",
                width: "100%",
                display: "block",
                disabled: cursor_state.read().is_grabbed(),
                onclick: move |_| {
                    *cursor_state.write() = CursorState::Grab;
                },
                if cursor_state.read().is_grabbed() {
                    "Press Esc to release cursor"
                } else {
                    "Capture cursor"
                }
            },
            for (dev, pos) in devices
                .iter()
                .filter(|d| d.selected)
                .zip(theremin_positions.read().iter().cloned())
            {
                div {
                    div {
                        "{dev.name}"
                    },
                    NoteBar {
                        note_width: 4.0, // TODO be able to change
                        note_scroll: pos,
                    },
                }
            }
        }
    }
}

#[component]
fn NoteBar(note_width: f32, note_scroll: f32) -> Element {
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
