use serde::{Serialize, Deserialize};
use super::instrument::Instrument;
use super::note_status::NoteStatus;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MidiPacket {
    pub pitch: u8,
    pub instrument: Instrument,
    pub note_status: NoteStatus,
    pub note_delta: f32,
    pub velocity: f32,
}
