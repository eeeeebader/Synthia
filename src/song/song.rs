use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Write, Read};
use super::midi_packet::MidiPacket;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Song {
    pub songname: String,
    pub artist: String,
    pub bpm: f32,
    pub packets: Vec<MidiPacket>,
}

// Save song to a JSON file
pub fn save_to_json(song: &Song, filename: &str) {
    let json = serde_json::to_string(song).unwrap();
    let mut file = File::create(filename).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}

// Load song from a JSON file
pub fn load_from_json(filename: &str) -> Song {
    let mut file = File::open(filename).unwrap();
    let mut json = String::new();
    file.read_to_string(&mut json).unwrap();
    serde_json::from_str(&json).unwrap()
}
