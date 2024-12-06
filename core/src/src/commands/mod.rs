type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub mod commands;
