use crate::midi::{Pitch, HIGHEST_MIDI_NOTE};

const DEFAULT_POS: f64 = 0.0; // TODO experiment with this value
const NOTE_WIDTH: f64 = 100.0; // TODO experiment with this value

pub struct InputHandler {
    pos: f64,
    max_pos: f64,
    note_boundaries: Vec<f64>,
    playing: bool,
}

impl InputHandler {
    pub fn new() -> Self {
        let note_boundaries = (0..(HIGHEST_MIDI_NOTE + 1))
            .map(|i| (i + 1) as f64 * NOTE_WIDTH)
            .collect();
        InputHandler {
            pos: DEFAULT_POS,
            max_pos: (HIGHEST_MIDI_NOTE + 1) as f64 * NOTE_WIDTH,
            note_boundaries,
            playing: false,
        }
    }

    pub fn reset(&mut self) {
        self.pos = DEFAULT_POS;
        self.playing = false;
    }

    pub fn handle_rel_move(&mut self, mov: f64) -> Pitch {
        self.pos = (self.pos + mov).clamp(0.0, self.max_pos);
        self.pitch_from_pos()
    }

    pub fn pitch_from_pos(&self) -> Pitch {
        for (i, bound) in self.note_boundaries.iter().enumerate() {
            if self.pos <= *bound {
                return i as Pitch;
            }
        }
        unreachable!("damn, my bad")
    }
}
