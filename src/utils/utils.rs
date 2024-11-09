use std::fs::File;
use std::io::Write;

pub fn save_vec_to_csv(data: Vec<f32>, filename: &str) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    for value in data {
        writeln!(file, "{}", value)?;
    }
    Ok(())
}
