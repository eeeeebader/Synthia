use serde::{Serialize, Deserialize};
use std::f32::consts::PI;
use rodio::{OutputStream, Source, buffer::SamplesBuffer};
use std::fs::File;
use std::io::{Write, Read};
use std::time::Duration;

// DEBUGGING
fn save_vec_to_csv(data: Vec<f32>, filename: &str) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    for value in data {
        writeln!(file, "{}", value)?;
    }
    Ok(())
}
// DEBUGGING END

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
enum Instrument {
    Sine,
    Square,
    Triangle,
    Saw,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
enum NoteStatus {
    On,
    Off,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct MidiPacket {
    pitch: u8,           // MIDI pitch (0-127)
    instrument: Instrument, 
    note_status: NoteStatus, 
    note_delta: f32,          // Time duration in beats
    velocity: f32,       // Volume (0.0 - 1.0)
}

// Function to generate a basic waveform
// TODO think about smoothing -> add a fade in and fade out to the waveform as lambda or however
fn generate_waveform(packet: &MidiPacket, sample_amount: usize, sample_rate: u32) -> Vec<f32> {
    let mut samples = Vec::new();
    let frequency = 440.0 * 2.0f32.powf((packet.pitch as f32 - 69.0) / 12.0);
    let amplitude = packet.velocity;

    for t in 0..sample_amount {
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


// Function to play the waveform
fn play_waveform(waveform: Vec<f32>, sample_rate: u32, duration: f32) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let source = SamplesBuffer::new(1, sample_rate, waveform);
    stream_handle.play_raw(source.convert_samples()).unwrap();
    
    // Keep the program running to allow the sound to play
    std::thread::sleep(Duration::from_secs((duration + 1f32) as u64));
}

fn generate_wave_from_packets(packets: &[MidiPacket], bpm: f32, sample_rate: u32) -> (f32, Vec<f32>) {
    let seconds_per_beat = 60.0 / bpm;
    let mut song_duration_sec = 0f32;
    for packet in packets {
        song_duration_sec += packet.note_delta * seconds_per_beat
    }

    let song_duration_samples = (song_duration_sec * sample_rate as f32) as usize;
    // create a vector to store the waveform
    let mut waveform = vec![0.0f32; song_duration_samples];

    let mut sample_index = 0;
    // iterate over the packets
    for (packet_index, packet) in packets.iter().enumerate() {
        // Get delta in samples
        let delta_samples = (packet.note_delta * seconds_per_beat * sample_rate as f32) as usize;

        sample_index += delta_samples;

        // skip if note off packet or last packet
        if packet.note_status == NoteStatus::Off || packet_index == packets.len() - 1 {
            continue;
        }

        let mut note_off_packet = None;
        // find note off packet and sum deltas for duration
        let mut note_duration_samples = 0;

        for next_packet in packets.iter().skip(packet_index + 1) {
            if packet.pitch == next_packet.pitch && packet.instrument == next_packet.instrument && next_packet.note_status == NoteStatus::Off {
                note_off_packet = Some(next_packet);
                note_duration_samples += (next_packet.note_delta * seconds_per_beat * sample_rate as f32) as usize;
                break;
            }
            note_duration_samples += (next_packet.note_delta * seconds_per_beat * sample_rate as f32) as usize;
        }
        // skip note if no note off packet found
        if note_off_packet.is_none() {
            continue;
        }

        // generate waveform for the note
        let note_waveform = generate_waveform(packet, note_duration_samples, sample_rate);

        // add the note waveform to the song waveform
        for (i, sample) in note_waveform.iter().enumerate() {
            if sample_index + i >= waveform.len() {
                break;
            }
            waveform[sample_index + i] += sample;
        }
    }

    // Normalize the waveform
    let max_amplitude = waveform.iter().copied().fold(0.0_f32, f32::max).max(1.0);
    for sample in &mut waveform {
        *sample /= max_amplitude;
    }

    (song_duration_sec, waveform)
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
        MidiPacket { pitch: 60, instrument: Instrument::Square, note_status: NoteStatus::On, note_delta: 0.0, velocity: 0.4 },
        //MidiPacket { pitch: 64, instrument: Instrument::Sine, note_status: NoteStatus::On, note_delta: 0.0, velocity: 0.4 },
        MidiPacket { pitch: 80, instrument: Instrument::Square, note_status: NoteStatus::On, note_delta: 10.0, velocity: 0.4 },
        MidiPacket { pitch: 40, instrument: Instrument::Square, note_status: NoteStatus::On, note_delta: 10.0, velocity: 0.4 },
        //MidiPacket { pitch: 72, instrument: Instrument::Sine, note_status: NoteStatus::On, note_delta: 0.0, velocity: 0.4 },
        MidiPacket { pitch: 60, instrument: Instrument::Square, note_status: NoteStatus::Off, note_delta: 2.0, velocity: 0.4 },
        //MidiPacket { pitch: 64, instrument: Instrument::Sine, note_status: NoteStatus::Off, note_delta: 0.0, velocity: 0.4 },
        MidiPacket { pitch: 80, instrument: Instrument::Square, note_status: NoteStatus::Off, note_delta: 10.0, velocity: 0.4 },
        MidiPacket { pitch: 40, instrument: Instrument::Square, note_status: NoteStatus::Off, note_delta: 0.0, velocity: 0.4 },
        //MidiPacket { pitch: 72, instrument: Instrument::Sine, note_status: NoteStatus::Off, note_delta: 0.0, velocity: 0.4 },
    ];

    // Save to JSON
    save_to_json(&packets, "testsong.json");

    // Load from JSON
    let loaded_packets = load_from_json("sweet_dreams.json");

    // Generate waveform and play
    let bpm = 60.0;
    let sample_rate = 44100;
    let (song_duration_secs, waveform) = generate_wave_from_packets(&loaded_packets, bpm, sample_rate);
    // write the waveform to a file
    save_vec_to_csv(waveform.clone(), "waveform.csv").unwrap();

    play_waveform(waveform, sample_rate, song_duration_secs);
}
