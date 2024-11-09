use crate::song::MidiPacket;
use crate::song::Instrument;
use crate::song::NoteStatus;

use std::f32::consts::PI;

pub fn generate_waveform(packet: &MidiPacket, sample_amount: usize, sample_rate: u32) -> Vec<f32> {
    let mut samples = Vec::new();
    let frequency = 440.0 * 2.0f32.powf((packet.pitch as f32 - 69.0) / 12.0);
    let amplitude = packet.velocity;

    for t in 0..sample_amount {
        let time = t as f32 / sample_rate as f32;

        let sample = match packet.instrument {
            Instrument::Sine => (2.0 * PI * frequency * time).sin(),
            Instrument::Square => if (2.0 * PI * frequency * time).sin() > 0.0 { 1.0 } else { -1.0 },
            Instrument::Triangle => (2.0 * PI * frequency * time).asin(),
            Instrument::Saw => 2.0 * ((frequency * time) % 1.0) - 1.0,
            Instrument::Piano => {  
                // Piano funciton by Inigo Quilez: https://www.youtube.com/watch?v=ogFAHvYatWs
                let decay_constant = -0.0015 * 2.0 * PI * frequency;

                // Base sine wave with exponential decay
                let mut piano_note = (2.0 * PI * frequency * time).sin() * (decay_constant * time).exp();

                // Add overtones with progressively halved amplitude
                piano_note += (2.0 * 2.0 * PI * frequency * time).sin() * (decay_constant * time).exp() / 2.0;
                piano_note += (3.0 * 2.0 * PI * frequency * time).sin() * (decay_constant * time).exp() / 4.0;
                piano_note += (4.0 * 2.0 * PI * frequency * time).sin() * (decay_constant * time).exp() / 8.0;
                piano_note += (5.0 * 2.0 * PI * frequency * time).sin() * (decay_constant * time).exp() / 16.0;
                piano_note += (6.0 * 2.0 * PI * frequency * time).sin() * (decay_constant * time).exp() / 32.0;

                // Apply saturation by adding cubic term for richness
                piano_note += piano_note * piano_note * piano_note;

                // Final envelope modulation for dynamic decay
                piano_note * (1.0 + 16.0 * time * (-6.0 * time).exp())
            },
        } * amplitude;

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
