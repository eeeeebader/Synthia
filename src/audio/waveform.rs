use crate::song::MidiPacket;
use crate::song::Instrument;
use crate::song::NoteStatus;

use std::f32::consts::PI;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Once;



static mut PROMINENT_FREQUENCIES: Option<Vec<(f32, f32)>> = None;
static INIT: Once = Once::new();

// Load and preprocess the prominent frequencies only once
fn load_prominent_frequencies() {
    // Read the CSV file
    let file = File::open("piano_overtones.csv").expect("Could not open file.");
    let reader = BufReader::new(file);

    let mut frequencies = Vec::new();

    // Skip the header row
    for (i, line) in reader.lines().enumerate() {
        if i == 0 { continue; }

        let line = line.expect("Could not read line.");
        let parts: Vec<&str> = line.split(',').collect();
        let frequency: f32 = parts[0].parse().expect("Could not parse frequency.");
        let amplitude: f32 = parts[1].parse().expect("Could not parse amplitude.");

        // Store relative frequency (as a multiplier of the base frequency) and amplitude
        frequencies.push((frequency, amplitude));
    }

    // Store the preloaded data in a global static variable
    unsafe {
        PROMINENT_FREQUENCIES = Some(frequencies);
    }
}

// Generate the piano sample by dynamically scaling the relative frequencies
fn generate_piano_sample(base_frequency: f32, time: f32) -> f32 {
    let base_decay_rate = -0.00015;          // Negative base decay rate
    let decay_threshold = 0.0005;    // Threshold below which we stop adding a frequency component

    // Ensure prominent frequencies are loaded only once
    INIT.call_once(|| {
        load_prominent_frequencies();
    });

    // Retrieve the preloaded prominent frequencies
    let prominent_frequencies = unsafe {
        PROMINENT_FREQUENCIES.as_ref().expect("Frequency data not loaded.")
    };

    let mut piano_note = 0.0;

    // Scale time to simulate a more noticeable decay effect
    let scaled_time = time * 50.0;  // Adjust this factor as needed

    for &(relative_freq, amp) in prominent_frequencies.iter() {
        let freq = relative_freq * base_frequency;

        // Adjust decay rate: base rate plus additional decay for higher frequencies
        // Calculate the decayed amplitude
        let decayed_amplitude = (base_decay_rate * freq * scaled_time).exp();

        // Skip this frequency if its contribution is below the threshold
        if decayed_amplitude.abs() < decay_threshold {
            break;
        }

        // Add the sine wave with the decayed amplitude to the overall piano note
        piano_note += amp * decayed_amplitude * (2.0 * PI * freq * time).sin();
    }

    piano_note  // Return the accumulated sample
}

pub fn generate_waveform(packet: &MidiPacket, sample_amount: usize, sample_rate: u32) -> Vec<f32> {
    let mut samples = Vec::new();
    let frequency = 440.0 * 2.0f32.powf((packet.pitch as f32 - 69.0) / 12.0);
    let amplitude = packet.velocity;

    let sample_amount_temp = (sample_amount as f32 * 1.5) as u32;

    for t in 0..sample_amount_temp {
        let time = t as f32 / sample_rate as f32;

        let sample = match packet.instrument {
            Instrument::Sine => (2.0 * PI * frequency * time).sin(),
            Instrument::Square => if (2.0 * PI * frequency * time).sin() > 0.0 { 1.0 } else { -1.0 },
            Instrument::Triangle => (2.0 * PI * frequency * time).asin(),
            Instrument::Saw => 2.0 * ((frequency * time) % 1.0) - 1.0,
            Instrument::Xylophone => {
                let decay_constant = -0.001 * 2.0 * PI * frequency;

                // Base sine wave with exponential decay
                let mut piano_note = (2.0 * PI * frequency * time).sin() * (decay_constant * time).exp();
                piano_note += (2.0 * PI * frequency * time).sin() * (decay_constant * time).exp();
                piano_note += (2.0 * PI * (frequency + 2.0) * time).sin() * (decay_constant * time).exp();

                piano_note /= 3.0;

                piano_note

            },
            Instrument::Piano => generate_piano_sample(frequency, time),
        } * amplitude;

        if t > 1000 && sample == 0.0 {
            break;
        }

        samples.push(sample);
    }

    samples
}

fn calculate_song_duration(packets: &[MidiPacket], bpm: f32, sample_rate: u32) -> (f32, usize) {
    let seconds_per_beat = 60.0 / bpm;
    let song_duration_sec: f32 = packets.iter().map(|packet| packet.note_delta * seconds_per_beat).sum();
    let song_duration_samples = (song_duration_sec * sample_rate as f32) as usize;
    (song_duration_sec, song_duration_samples)
}

fn calculate_note_duration(packets: &[MidiPacket], start_index: usize, bpm: f32, sample_rate: u32) -> Option<usize> {
    let seconds_per_beat = 60.0 / bpm;
    let mut note_duration_samples = 0;

    for next_packet in packets.iter().skip(start_index + 1) {
        note_duration_samples += (next_packet.note_delta * seconds_per_beat * sample_rate as f32) as usize;
        if next_packet.pitch == packets[start_index].pitch
            && next_packet.instrument == packets[start_index].instrument
            && next_packet.note_status == NoteStatus::Off
        {
            return Some(note_duration_samples);
        }
    }

    None
}

fn add_note_waveform(waveform: &mut Vec<f32>, note_waveform: &[f32], start_index: usize) {
    for (i, sample) in note_waveform.iter().enumerate() {
        if start_index + i >= waveform.len() {
            break;
        }
        waveform[start_index + i] += sample;
    }
}

fn normalize_waveform(waveform: &mut Vec<f32>) {
    let max_amplitude = waveform.iter().copied().fold(0.0_f32, f32::max).max(1.0);
    if max_amplitude > 0.0 {
        for sample in waveform {
            *sample /= max_amplitude;
        }
    }
}

pub fn generate_wave_from_packets(packets: &[MidiPacket], bpm: f32, sample_rate: u32) -> (f32, Vec<f32>) {
    // Calculate song duration
    let (song_duration_sec, song_duration_samples) = calculate_song_duration(packets, bpm, sample_rate);
    let mut waveform = vec![0.0f32; song_duration_samples];

    // Process each packet
    let seconds_per_beat = 60.0 / bpm;
    let mut sample_index = 0;

    for (packet_index, packet) in packets.iter().enumerate() {
        sample_index += (packet.note_delta * seconds_per_beat * sample_rate as f32) as usize;

        // Skip if note is off or it's the last packet
        if packet.note_status == NoteStatus::Off || packet_index == packets.len() - 1 {
            continue;
        }

        // Calculate the duration of the current note
        let note_duration_samples = match calculate_note_duration(packets, packet_index, bpm, sample_rate) {
            Some(duration) => duration,
            None => continue,
        };

        // Generate the waveform for the note
        let note_waveform = generate_waveform(packet, note_duration_samples, sample_rate);

        // Add note waveform to the main song waveform
        add_note_waveform(&mut waveform, &note_waveform, sample_index);
    }

    // Normalize the waveform
    normalize_waveform(&mut waveform);


    (song_duration_sec, waveform)
}
