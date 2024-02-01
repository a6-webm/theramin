use std::{env, os::fd::AsRawFd};

use evdev::{Device, FetchEventsSynced, InputEventKind, Key};
use libc::{F_SETFL, O_NONBLOCK};

struct InputParams {
    lowest_pitch: usize,
    num_notes: usize,
    note_width: usize,
}

fn main() {
    let params = InputParams {
        num_notes: 3 * 12,
        note_width: 250,
        lowest_pitch: 44,
    };
    let args: Vec<String> = env::args().collect();
    let mut mouse = Device::open(args[1].clone()).unwrap();
    assert!(unsafe { libc::fcntl(mouse.as_raw_fd(), F_SETFL, O_NONBLOCK) } == 0);

    println!("playing off, right click to start playing");
    loop {
        main_loop(&mut mouse);
    }
}

fn main_loop(mouse: &mut Device) {
    loop {
        off_loop(mouse);
        wait_for_hands_off_mouse(mouse);
        on_loop(mouse);
        wait_for_hands_off_mouse(mouse);
    }
}

fn off_loop(mouse: &mut Device) {
    loop {
        match get_evdev_events(mouse) {
            Some(ev_iter) => {
                for ev in ev_iter {
                    if ev.kind() == InputEventKind::Key(Key::BTN_RIGHT) && ev.value() == 1 {
                        return;
                    }
                }
            }
            None => continue,
        }
    }
}

fn on_loop(mouse: &mut Device) {
    loop {
        match get_evdev_events(mouse) {
            Some(ev_iter) => {
                for ev in ev_iter {
                    match ev.kind() {
                        InputEventKind::Key(Key::BTN_RIGHT)
                        | InputEventKind::Key(Key::BTN_TOUCH) => todo!("play or stop notes"),
                        InputEventKind::RelAxis(_) => handle_rel_move(),
                        InputEventKind::AbsAxis(_) => handle_abs_move(),
                        _ => continue,
                    }
                }
            }
            None => continue,
        }
    }
}

fn handle_abs_move() {
    todo!()
}

fn handle_rel_move() {
    todo!()
}

fn get_evdev_events(dev: &mut Device) -> Option<FetchEventsSynced> {
    match dev.fetch_events() {
        Ok(ev_iter) => Some(ev_iter),
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => None,
        Err(e) => panic!("{}", e),
    }
}

fn wait_for_hands_off_mouse(mouse: &mut Device) {
    loop {
        match get_evdev_events(mouse) {
            Some(ev_iter) => for _ in ev_iter {},
            None => continue,
        }
        // debug
        {
            let pressed_sus = mouse.cached_state().key_vals().unwrap();
            println!("Pressed keys: {:?}", pressed_sus);
        }
        if mouse.cached_state().key_vals().unwrap().iter().count() == 0 {
            return;
        }
    }
}
