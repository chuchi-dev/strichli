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
		// the details of an internal error stay in the log
		let this = match self {
			Self::Internal(detail) => {
				tracing::error!("internal error: {detail}");
				Self::Internal("see the server log".into())
			}
			other => other,
		};
		let status = this.status_code();
		let retry_after = match &this {
			Self::LimitReached(secs) => Some(*secs),
			_ => None,
		};

		let mut resp = (
			status,
			Json(ErrorResponse {
				message: this.to_string(),
				error: this,
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
		match e {
			// a taken email or domain
			pg::Error::UniqueViolation { constraint, .. } => Self::Conflict(
				match constraint.as_deref() {
					Some("email_unique") => "this email already has an account",
					Some("domain_unique") => "this site already exists",
					_ => "this already exists",
				}
				.into(),
			),
			e => Self::Internal(e.to_string()),
		}
	}
}

impl From<pg::database::DatabaseError> for Error {
	fn from(e: pg::database::DatabaseError) -> Self {
		Self::Internal(e.to_string())
	}
}
