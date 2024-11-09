use rodio::{OutputStream, Source, buffer::SamplesBuffer};
use std::time::Duration;

pub fn play_waveform(waveform: Vec<f32>, sample_rate: u32, duration: f32) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let source = SamplesBuffer::new(1, sample_rate, waveform);
    stream_handle.play_raw(source.convert_samples()).unwrap();

    std::thread::sleep(Duration::from_secs((duration + 1f32) as u64));
}
