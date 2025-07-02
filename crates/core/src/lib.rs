use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("No batteries found in {}", .path)]
    NoBatteriesFound { path: String },
    #[error("Invalid path: {}", .path)]
    InvalidPath { path: String },
}
