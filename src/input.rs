use crate::midi::{Pitch, HIGHEST_MIDI_NOTE};

pub struct InputHandler {
    pos: u32,
    max_pos: u32,
    note_boundaries: Vec<u32>,
    playing: bool,
}

impl InputHandler {
    pub fn new(note_width: u16) -> Self {
        let note_boundaries = (0..(HIGHEST_MIDI_NOTE + 1))
            .map(|i| (i + 1) as u32 * note_width as u32)
            .collect();
        let max_pos = (HIGHEST_MIDI_NOTE + 1) as u32 * note_width as u32;
        InputHandler {
            pos: max_pos / 2,
            max_pos,
            note_boundaries,
            playing: false,
        }
    }

    pub fn reset(&mut self) {
        self.pos = self.max_pos / 2;
        self.playing = false;
    }

    fn handle_rel_move(&mut self, mov: i32) -> Pitch {
        if mov > 0 {
            self.pos = (self.pos + mov as u32).min(self.max_pos - 1);
        } else {
            let mov = mov.abs() as u32;
            if mov > self.pos {
                self.pos = 0;
            } else {
                self.pos = self.pos - mov;
            }
        }
        self.pitch_from_pos()
    }

    pub fn pitch_from_pos(&self) -> Pitch {
        for (i, bound) in self.note_boundaries.iter().enumerate() {
            if self.pos < *bound {
                return i as Pitch;
            }
        }
        unreachable!("damn, my bad")
    }
}
