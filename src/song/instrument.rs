use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum Instrument {
    Sine,
    Square,
    Triangle,
    Saw,
    Piano,
}
