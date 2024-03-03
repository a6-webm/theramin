#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use std::io::{self, Write};
use theramin::{
    input::InputHandler,
    manymouse::{self, Axis, Button, ManyMouse},
    midi::MidiInitialiser,
};

fn main() {
    // simple_logger::init_with_level(log::Level::Debug).unwrap();
    dioxus_desktop::launch_cfg(
        App,
        Config::default().with_window(WindowBuilder::new().with_inner_size(
            dioxus_desktop::wry::application::dpi::LogicalSize::new(400.0, 800.0),
        )),
    );
}

#[component]
fn App(cx: Scope) -> Element {
    use_on_create(cx, || {
        tokio::spawn(async move {
            let mut m_mouse = ManyMouse::new();
            println!("{}", m_mouse.driver_name());
            println!("{:?}", m_mouse.device_list());

            let mut buffer = String::new();
            let stdin = io::stdin();
            print!("Which dev bruh: ");
            io::stdout().flush().unwrap();
            stdin.read_line(&mut buffer).unwrap();
            let dev_idx: u32 = buffer.trim().parse().unwrap();

            let mut midi_h = MidiInitialiser::new().virtual_port("port_1");

            let mut input_h = InputHandler::new(50);

            loop {
                for ev in m_mouse.poll() {
                    println!("{:?}", ev);
                    if ev.device != dev_idx {
                        continue;
                    }
                    match ev.ev_type {
                        manymouse::EventType::Relmotion if ev.item == Button::LMB as u32 => {
                            let pitch = input_h.handle_rel_move(ev.value);
                            if input_h.playing {
                                midi_h.play(pitch);
                            }
                        }
                        manymouse::EventType::Button if ev.item == Axis::X as u32 => {
                            input_h.playing = ev.value == 1;
                            if input_h.playing {
                                midi_h.play(input_h.pitch_from_pos());
                            } else {
                                midi_h.release();
                            }
                        }
                        _ => (),
                    }
                }
            }
        })
    });
    cx.render(rsx!("hello world!"))
}
