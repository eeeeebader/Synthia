mod song;
mod audio;
mod utils;

use audio::generate_wave_from_packets;
use audio::play_waveform;
use utils::save_vec_to_csv;
use song::load_from_json;

fn main() {
    let filename_in: &str = "piano.json";
    let filename_out: &str = &(format!("{}.csv", filename_in.split('.').next().unwrap()));

    let loaded_song = load_from_json(filename_in);

    let sample_rate = 44100;
    let (song_duration_secs, waveform) = generate_wave_from_packets(&loaded_song.packets, loaded_song.bpm, sample_rate);

    save_vec_to_csv(waveform.clone(), filename_out).unwrap();
    play_waveform(waveform, sample_rate, song_duration_secs);
}
