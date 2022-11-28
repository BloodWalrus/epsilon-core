pub mod connection;
pub mod constants;

pub type EpsilonResult<T> = Result<T, Box<dyn std::error::Error>>;
