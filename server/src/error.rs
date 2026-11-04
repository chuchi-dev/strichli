use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error, Clone, Serialize)]
#[serde(tag = "type", content = "detail", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Error {
	#[error("{0}")]
	Invalid(String),
	#[error("Rate limit reached, try again in {0}s")]
	LimitReached(u64),
	#[error("Wrong email or password")]
	LoginIncorrect,
	#[error("The session token does not exist or the session is expired")]
	InvalidSessionToken,
	#[error("The api token does not exist")]
	InvalidApiToken,
	#[error("Only admins can do this")]
	AdminOnly,
	#[error("Not found")]
	NotFound,
	#[error("{0}")]
	Conflict(String),
	#[error("Internal server error: {0}")]
	Internal(String),
}

impl Error {
	pub fn status_code(&self) -> StatusCode {
		match self {
			Self::Invalid(_) => StatusCode::BAD_REQUEST,
			Self::LimitReached(_) => StatusCode::TOO_MANY_REQUESTS,
			Self::LoginIncorrect
			| Self::InvalidSessionToken
			| Self::InvalidApiToken => StatusCode::UNAUTHORIZED,
			Self::AdminOnly => StatusCode::FORBIDDEN,
			Self::NotFound => StatusCode::NOT_FOUND,
			Self::Conflict(_) => StatusCode::CONFLICT,
			Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
	message: String,
	error: Error,
}

impl IntoResponse for Error {
	fn into_response(self) -> Response {
		let status = self.status_code();
		let retry_after = match &self {
			Self::LimitReached(secs) => Some(*secs),
			_ => None,
		};

		let mut resp = (
			status,
			Json(ErrorResponse {
				message: self.to_string(),
				error: self,
			}),
		)
			.into_response();

		if let Some(secs) = retry_after {
			resp.headers_mut()
				.insert(header::RETRY_AFTER, HeaderValue::from(secs));
		}

		resp
	}
}

impl From<pg::Error> for Error {
	fn from(e: pg::Error) -> Self {
		Self::Internal(e.to_string())
	}
}

impl From<pg::database::DatabaseError> for Error {
	fn from(e: pg::database::DatabaseError) -> Self {
		Self::Internal(e.to_string())
	}
}
