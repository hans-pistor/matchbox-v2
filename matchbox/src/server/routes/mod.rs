pub mod error;
pub mod sandbox;

pub type ApiResult<T> = Result<T, error::ApiError>;
