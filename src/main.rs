use serde::{Serialize, Deserialize};
use std::f32::consts::PI;
use rodio::{OutputStream, Source, buffer::SamplesBuffer};
use std::fs::File;
use std::io::{Write, Read};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Instrument {
    Sine,
    Square,
    Triangle,
    Saw,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum NoteStatus {
    On,
    Off,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MidiPacket {
    pitch: u8,           // MIDI pitch (0-127)
    instrument: Instrument, 
    note_status: NoteStatus, 
    delta: f32,          // Time duration in beats
    velocity: f32,       // Volume (0.0 - 1.0)
}

// Function to generate a basic waveform
fn generate_waveform(packet: &MidiPacket, sample_rate: u32, duration: f32) -> Vec<f32> {
    let mut samples = Vec::new();
    let frequency = 440.0 * 2.0f32.powf((packet.pitch as f32 - 69.0) / 12.0);
    let amplitude = packet.velocity;

    for t in 0..(sample_rate as f32 * duration) as usize {
        let sample = match packet.instrument {
            Instrument::Sine => (2.0 * PI * frequency * t as f32 / sample_rate as f32).sin(),
            Instrument::Square => if (2.0 * PI * frequency * t as f32 / sample_rate as f32).sin() > 0.0 { 1.0 } else { -1.0 },
            Instrument::Triangle => (2.0 * PI * frequency * t as f32 / sample_rate as f32).asin(),
            Instrument::Saw => 2.0 * ((frequency * t as f32 / sample_rate as f32) % 1.0) - 1.0,
        } * amplitude;
        samples.push(sample);
    }

    samples
}

// Function to mix multiple waveforms for polyphony
fn mix_waveforms(packets: &[MidiPacket], bpm: f32, sample_rate: u32) -> Vec<f32> {
    let mut mixed_waveform = Vec::new();
    let seconds_per_beat = 60.0 / bpm;

    for packet in packets {
        let duration = packet.delta * seconds_per_beat;
        let waveform = generate_waveform(packet, sample_rate, duration);
        
        // Resize the mixed_waveform vector if necessary
        if mixed_waveform.len() < waveform.len() {
            mixed_waveform.resize(waveform.len(), 0.0);
        }

        // Add the waveform to the mix (polyphony)
        for (i, sample) in waveform.iter().enumerate() {
            mixed_waveform[i] += sample;
        }
    }

    // Normalize the waveform
    let max_amplitude = mixed_waveform.iter().copied().fold(0.0_f32, f32::max).max(1.0);
    for sample in &mut mixed_waveform {
        *sample /= max_amplitude;
    }

    mixed_waveform
}

// Function to play the waveform
fn play_waveform(waveform: Vec<f32>, sample_rate: u32) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let source = SamplesBuffer::new(1, sample_rate, waveform);
    stream_handle.play_raw(source.convert_samples()).unwrap();
    
    // Keep the program running to allow the sound to play
    std::thread::sleep(Duration::from_secs(5));
}

// Save packets to a JSON file
fn save_to_json(packets: &[MidiPacket], filename: &str) {
    let json = serde_json::to_string(packets).unwrap();
    let mut file = File::create(filename).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}

// Load packets from a JSON file
fn load_from_json(filename: &str) -> Vec<MidiPacket> {
    let mut file = File::open(filename).unwrap();
    let mut json = String::new();
    file.read_to_string(&mut json).unwrap();
    serde_json::from_str(&json).unwrap()
}

fn main() {
    let packets = vec![
        MidiPacket { pitch: 60, instrument: Instrument::Square, note_status: NoteStatus::On, delta: 0.0, velocity: 0.4 },
        //MidiPacket { pitch: 64, instrument: Instrument::Sine, note_status: NoteStatus::On, delta: 0.0, velocity: 0.4 },
        MidiPacket { pitch: 80, instrument: Instrument::Square, note_status: NoteStatus::On, delta: 10.0, velocity: 0.4 },
        MidiPacket { pitch: 40, instrument: Instrument::Square, note_status: NoteStatus::On, delta: 10.0, velocity: 0.4 },
        //MidiPacket { pitch: 72, instrument: Instrument::Sine, note_status: NoteStatus::On, delta: 0.0, velocity: 0.4 },
        MidiPacket { pitch: 60, instrument: Instrument::Square, note_status: NoteStatus::Off, delta: 2.0, velocity: 0.4 },
        //MidiPacket { pitch: 64, instrument: Instrument::Sine, note_status: NoteStatus::Off, delta: 0.0, velocity: 0.4 },
        MidiPacket { pitch: 80, instrument: Instrument::Square, note_status: NoteStatus::Off, delta: 10.0, velocity: 0.4 },
        MidiPacket { pitch: 40, instrument: Instrument::Square, note_status: NoteStatus::Off, delta: 0.0, velocity: 0.4 },
        //MidiPacket { pitch: 72, instrument: Instrument::Sine, note_status: NoteStatus::Off, delta: 0.0, velocity: 0.4 },
    ];

    // Save to JSON
    save_to_json(&packets, "song.json");

    // Load from JSON
    let loaded_packets = load_from_json("song.json");

    // Generate waveform and play
    let bpm = 60.0;
    let sample_rate = 44100;
    let waveform = mix_waveforms(&loaded_packets, bpm, sample_rate);
    play_waveform(waveform, sample_rate);
}
