#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("todo")]
    Todo,
}

pub type Result<T> = std::result::Result<T, Error>;
