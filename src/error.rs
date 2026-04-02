pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid rule")]
    InvalidRule,
    #[error("Invalid image")]
    InvalidImage,
    #[error("Invalid data length")]
    InvalidLength,
}
