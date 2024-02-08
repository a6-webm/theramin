// TODO notes playing multiple for some reason

use std::{os::fd::AsRawFd, path::PathBuf};

use clap::Parser;
use evdev::{AbsoluteAxisType, Device, FetchEventsSynced, InputEventKind, Key, RelativeAxisType};
use libc::{F_SETFL, O_NONBLOCK};
use midir::{os::unix::VirtualOutput, MidiOutput, MidiOutputConnection};

const HIGHEST_MIDI_NOTE: u8 = 127;
const VEL: u8 = 127;
const NOTE_ON_MSG: u8 = 0x90;
const NOTE_OFF_MSG: u8 = 0x80;

type Pitch = u8;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct InputParams {
    /// The evdev file of your mouse
    #[arg(value_name = "DEVICE_PATH")]
    dev_path: PathBuf,
    /// The lowest midi note you want to play
    #[arg(value_name = "LOWEST_PITCH")]
    lowest_pitch: Pitch,
    /// The number of notes you want your theremin to have
    #[arg(value_name = "NUM_NOTES")]
    num_notes: u8,
    /// The width of each note in mouse-space units
    #[arg(value_name = "NOTE_WIDTH")]
    note_width: usize,
}

struct InputHandler {
    pos: usize,
    max_pos: usize,
    note_boundaries: Vec<usize>,
    playing: bool,
    params: InputParams,
}

impl InputHandler {
    fn new(params: InputParams) -> Self {
        assert!(params.num_notes >= 1);
        assert!(params.lowest_pitch + params.num_notes <= HIGHEST_MIDI_NOTE + 1);
        let note_boundaries = (0..params.num_notes)
            .map(|i| (i + 1) as usize * params.note_width)
            .collect();
        InputHandler {
            pos: 0,
            max_pos: params.num_notes as usize * params.note_width,
            note_boundaries,
            params,
            playing: false,
        }
    }

    fn reset(&mut self) {
        self.pos = 0;
        self.playing = false;
    }

    fn handle_rel_move(&mut self, mov: i32) -> Pitch {
        if mov > 0 {
            self.pos = (self.pos + mov as usize).min(self.max_pos);
        } else {
            let mov = mov.abs() as usize;
            if mov > self.pos {
                self.pos = 0;
            } else {
                self.pos = self.pos - mov;
            }
        }
        self.pitch_from_pos()
    }

    fn handle_abs_move(&mut self, pos: i32) -> Pitch {
        self.pos = (pos as usize).min(self.max_pos - 1);
        self.pitch_from_pos()
    }

    fn pitch_from_pos(&self) -> Pitch {
        for (i, bound) in self.note_boundaries.iter().enumerate() {
            if self.pos < *bound {
                return i as u8 + self.params.lowest_pitch;
            }
        }
        panic!("position more than maximum (not sure how you've done this)");
    }
}

struct MidiHandler {
    current_note: Option<Pitch>,
    conn_out: MidiOutputConnection,
}

impl MidiHandler {
    fn new() -> Self {
        let midi_out = MidiOutput::new("Theramin_midi_out").unwrap();
        let conn_out = midi_out.create_virtual("Theramin").unwrap();
        MidiHandler {
            current_note: None,
            conn_out,
        }
    }

    fn play(&mut self, pitch: Pitch) {
        match self.current_note {
            Some(current_note) if current_note == pitch => return,
            Some(current_note) => {
                self.conn_out
                    .send(&[NOTE_OFF_MSG, current_note, VEL])
                    .unwrap();
            }
            None => (),
        }
        self.conn_out.send(&[NOTE_ON_MSG, pitch, VEL]).unwrap();
        self.current_note = Some(pitch);
    }

    fn release(&mut self) {
        if let Some(current_note) = self.current_note {
            self.conn_out
                .send(&[NOTE_OFF_MSG, current_note, VEL])
                .unwrap();
            self.current_note = None;
        }
    }
}

fn main() {
    let input_params = InputParams::parse();

    let mut mouse = Device::open(input_params.dev_path.clone()).unwrap();
    assert!(unsafe { libc::fcntl(mouse.as_raw_fd(), F_SETFL, O_NONBLOCK) } == 0);

    let mut midi_h = MidiHandler::new();

    let mut input_h = InputHandler::new(input_params);

    loop {
        main_loop(&mut mouse, &mut midi_h, &mut input_h);
    }
}

fn main_loop(mouse: &mut Device, midi_h: &mut MidiHandler, input_h: &mut InputHandler) {
    loop {
        off_loop(mouse);
        wait_for_hands_off_mouse(mouse);
        mouse.grab().unwrap();
        input_h.reset();
        on_loop(mouse, midi_h, input_h);
        wait_for_hands_off_mouse(mouse);
        mouse.ungrab().unwrap();
        midi_h.release();
    }
}

fn off_loop(mouse: &mut Device) {
    println!("playing off, right click or tap with three fingers to start playing");
    loop {
        match get_evdev_events(mouse) {
            Some(ev_iter) => {
                for ev in ev_iter {
                    if (ev.kind() == InputEventKind::Key(Key::BTN_RIGHT)
                        || ev.kind() == InputEventKind::Key(Key::BTN_TOOL_TRIPLETAP))
                        && ev.value() == 1
                    {
                        return;
                    }
                }
            }
            None => continue,
        }
    }
}

// TODO touch event always comes before its position on a trackpad, so need to process moving before pressing somehow
fn on_loop(mouse: &mut Device, midi_h: &mut MidiHandler, input_h: &mut InputHandler) {
    println!("Left click or touch the touchpad to play notes, right click or tap with three fingers to stop");
    loop {
        match get_evdev_events(mouse) {
            Some(ev_iter) => {
                for ev in ev_iter {
                    match ev.kind() {
                        InputEventKind::Key(Key::BTN_LEFT)
                        | InputEventKind::Key(Key::BTN_TOUCH) => {
                            input_h.playing = ev.value() == 1;
                            if input_h.playing {
                                midi_h.play(input_h.pitch_from_pos());
                            } else {
                                midi_h.release();
                            }
                        }
                        InputEventKind::RelAxis(RelativeAxisType::REL_X) => {
                            let pitch = input_h.handle_rel_move(ev.value());
                            if input_h.playing {
                                midi_h.play(pitch);
                            }
                        }
                        InputEventKind::AbsAxis(AbsoluteAxisType::ABS_X) => {
                            let pitch = input_h.handle_abs_move(ev.value());
                            if input_h.playing {
                                midi_h.play(pitch);
                            }
                        }
                        InputEventKind::Key(Key::BTN_RIGHT)
                        | InputEventKind::Key(Key::BTN_TOOL_TRIPLETAP)
                            if ev.value() == 1 =>
                        {
                            return
                        }
                        _ => continue,
                    }
                }
            }
            None => continue,
        }
    }
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
        if mouse.cached_state().key_vals().unwrap().iter().count() == 0 {
            return;
        }
    }
}
