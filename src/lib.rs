pub mod connection;

pub type EpsilonResult<T> = Result<T, Box<dyn std::error::Error>>;
