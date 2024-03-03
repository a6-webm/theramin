use midir::{os::unix::VirtualOutput, MidiOutput, MidiOutputConnection, MidiOutputPort};

pub const HIGHEST_MIDI_NOTE: u8 = 127;
const VEL: u8 = 127;
const NOTE_ON_MSG: u8 = 0x90;
const NOTE_OFF_MSG: u8 = 0x80;

pub type Pitch = u8;

pub struct MidiInitialiser {
    midi_out: MidiOutput,
}

pub struct MidiHandler {
    current_note: Option<Pitch>,
    conn_out: MidiOutputConnection,
}

impl MidiInitialiser {
    pub fn new() -> Self {
        MidiInitialiser {
            midi_out: MidiOutput::new("Theramin midi out").unwrap(),
        }
    }

    pub fn from_output(midi_out: MidiOutput) -> Self {
        MidiInitialiser { midi_out }
    }

    pub fn get_ports(&self) -> Vec<(String, MidiOutputPort)> {
        self.midi_out
            .ports()
            .into_iter()
            .map(|port| (self.midi_out.port_name(&port).unwrap(), port))
            .collect()
    }

    pub fn virtual_port(self, name: &str) -> MidiHandler {
        MidiHandler::new(self.midi_out.create_virtual(name).unwrap())
    }

    pub fn connect(self, port: (String, &MidiOutputPort)) -> MidiHandler {
        MidiHandler::new(self.midi_out.connect(port.1, &port.0).unwrap())
    }
}

impl MidiHandler {
    fn new(conn_out: MidiOutputConnection) -> Self {
        MidiHandler {
            current_note: None,
            conn_out,
        }
    }

    pub fn play(&mut self, pitch: Pitch) {
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

    pub fn release(&mut self) {
        if let Some(current_note) = self.current_note {
            self.conn_out
                .send(&[NOTE_OFF_MSG, current_note, VEL])
                .unwrap();
            self.current_note = None;
        }
    }

    pub fn close(mut self) -> MidiInitialiser {
        self.release();
        MidiInitialiser::from_output(self.conn_out.close())
    }
}
