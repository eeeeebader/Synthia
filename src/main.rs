mod song;
mod audio;
mod utils;

use audio::generate_wave_from_packets;
use audio::play_waveform;
use utils::save_vec_to_csv;
use song::load_from_json;

fn main() {
    let loaded_song = load_from_json("sweet_dreams.json");

    let sample_rate = 44100;
    let (song_duration_secs, waveform) = generate_wave_from_packets(&loaded_song.packets, loaded_song.bpm, sample_rate);

    save_vec_to_csv(waveform.clone(), "sweet_dreams.csv").unwrap();
    play_waveform(waveform, sample_rate, song_duration_secs);
}
