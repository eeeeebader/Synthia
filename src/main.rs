use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Read};

use serde_json;

const SAMPLE_RATE: f32 = 44100.0;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Instrument {
    Sine,
    Saw,
    Square,
    Triangle,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct NotePacket {
    pitch: u8,
    instrument: Instrument,
    note_length: f32,
    delay: f32,
    velocity: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct Song {
    bpm: u32,
    packets: Vec<NotePacket>,
}

impl Song {
    pub fn new(bpm: u32) -> Self {
        Self {
            bpm,
            packets: Vec::new(),
        }
    }

    pub fn add_note(&mut self, packet: NotePacket) {
        self.packets.push(packet);
    }

    pub fn save_to_file(&self, filename: &str) -> io::Result<()> {
        let file = File::create(filename)?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }

    pub fn load_from_file(filename: &str) -> io::Result<Self> {
        let mut file = File::open(filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let song: Song = serde_json::from_str(&contents)?;
        Ok(song)
    }
}

fn midi_to_frequency(midi_note: u8) -> f32 {
    440.0 * 2.0_f32.powf((midi_note as f32 - 69.0) / 12.0)
}

fn generate_wave(packet: &NotePacket, bpm: u32) -> Vec<f32> {
    let frequency = midi_to_frequency(packet.pitch);
    let duration_in_seconds = (60.0 / bpm as f32) * packet.note_length;
    let num_samples = (duration_in_seconds * SAMPLE_RATE) as usize;
    let mut waveform = Vec::new();

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE;
        let sample = match packet.instrument {
            Instrument::Sine => (2.0 * std::f32::consts::PI * frequency * t).sin(),
            Instrument::Saw => 2.0 * (t * frequency - (t * frequency).floor()) - 1.0,
            Instrument::Square => {
                if (t * frequency).sin() > 0.0 {
                    1.0
                } else {
                    -1.0
                }
            }
            Instrument::Triangle => 2.0 * (2.0 * (t * frequency - 0.5).abs() - 1.0).abs() - 1.0,
        } * (packet.velocity as f32 / 127.0);

        waveform.push(sample);
    }

    waveform
}

// New function to mix multiple waveforms together, supporting polyphony.
fn mix_waveforms(waveforms: Vec<Vec<f32>>) -> Vec<f32> {
    let max_length = waveforms.iter().map(|w| w.len()).max().unwrap_or(0);
    let mut mixed = vec![0.0; max_length];

    for wave in waveforms {
        for (i, sample) in wave.iter().enumerate() {
            if i < mixed.len() {
                mixed[i] += sample;
            }
        }
    }

    // Normalize the mixed waveform to avoid clipping
    let max_sample = mixed.iter().cloned().fold(0.0_f32, f32::max);
    if max_sample > 1.0 {
        for sample in &mut mixed {
            *sample /= max_sample;
        }
    }

    mixed
}

// Updated function to generate a polyphonic song waveform
fn generate_song_waveform(song: &Song) -> Vec<f32> {
    let mut active_notes: Vec<(Vec<f32>, usize)> = Vec::new(); // (waveform, samples left)
    let mut song_waveform = Vec::new();
    let delay_samples =
        |delay: f32, bpm: u32| -> usize { ((60.0 / bpm as f32) * delay * SAMPLE_RATE) as usize };

    let mut current_time = 0;

    for packet in &song.packets {
        let start_samples = delay_samples(packet.delay, song.bpm);
        if start_samples > current_time {
            // Extend the buffer up to the start time of the new note
            let silence_duration = start_samples - current_time;
            song_waveform.extend(std::iter::repeat(0.0).take(silence_duration));
            current_time = start_samples;
        }

        // Add the new note
        let wave = generate_wave(packet, song.bpm);
        let wave_len = wave.len();
        active_notes.push((wave, wave_len));

        // Mix active notes and add to the song waveform
        let mixed_wave = mix_waveforms(active_notes.iter().map(|(w, _)| w.clone()).collect());
        let mixed_wave_len = mixed_wave.len();

        song_waveform.extend(mixed_wave);
        // Reduce the duration of active notes
        active_notes = active_notes
            .into_iter()
            .map(|(w, samples)| (w, samples.saturating_sub(mixed_wave_len)))
            .filter(|(_, samples)| *samples > 0)
            .collect();

        current_time += mixed_wave_len;
    }

    song_waveform
}

fn play_waveform(waveform: Vec<f32>) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // Convert the f32 samples to i16
    let samples: Vec<i16> = waveform
        .iter()
        .map(|&x| (x * i16::MAX as f32) as i16)
        .collect();

    // Directly create a SamplesBuffer source with the samples
    let source = SamplesBuffer::new(1, SAMPLE_RATE as u32, samples);

    // Append the source to the sink and play
    sink.append(source);
    sink.sleep_until_end();
}

fn main() {
    let mut song = Song::new(60);

    // Example of adding notes with overlapping timings for polyphony
    song.add_note(NotePacket {
        pitch: 60,
        instrument: Instrument::Sine,
        note_length: 4.0,
        delay: 0.0,
        velocity: 255,
    });

    song.add_note(NotePacket {
        pitch: 64,
        instrument: Instrument::Sine,
        note_length: 4.0,
        delay: 0.0,
        velocity: 255,
    });

    song.add_note(NotePacket {
        pitch: 67,
        instrument: Instrument::Sine,
        note_length: 4.0,
        delay: 0.0,
        velocity: 255,
    });

    song.add_note(NotePacket {
        pitch: 72,
        instrument: Instrument::Sine,
        note_length: 4.0,
        delay: 0.0,
        velocity: 255,
    });

    song.save_to_file("polyphonic_song.json").unwrap();

    let loaded_song = Song::load_from_file("polyphonic_song.json").unwrap();
    let waveform = generate_song_waveform(&loaded_song);

    play_waveform(waveform);
}
