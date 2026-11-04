//! Security headers for the webui

use axum::extract::State;
use axum::http::{HeaderName, HeaderValue, header};
use axum::response::Response;

const CSP: &str = "\
default-src 'self'; \
script-src 'self'; \
style-src 'self' 'unsafe-inline'; \
img-src 'self' data:; \
connect-src 'self'";

const HEADERS: &[(HeaderName, &str)] = &[
	(header::STRICT_TRANSPORT_SECURITY, "max-age=31536000"),
	(header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
	(header::REFERRER_POLICY, "same-origin"),
];

/// The headers that depend on the config
#[derive(Debug, Clone)]
pub struct SecurityHeaders {
	csp: HeaderValue,
	/// only without allowed frame ancestors, it can't list origins
	deny_frames: bool,
}

impl SecurityHeaders {
	/// `frame_ancestors` are the origins that may embed the webui, none by
	/// default
	pub fn new(frame_ancestors: &[String]) -> Self {
		let ancestors = if frame_ancestors.is_empty() {
			"'none'".to_string()
		} else {
			frame_ancestors.join(" ")
		};

		Self {
			csp: HeaderValue::from_str(&format!(
				"{CSP}; frame-ancestors {ancestors}"
			))
			.expect("invalid frame-ancestors in config.toml"),
			deny_frames: frame_ancestors.is_empty(),
		}
	}
}

/// Use with `axum::middleware::map_response_with_state`
pub async fn security_headers(
	State(sec): State<SecurityHeaders>,
	mut resp: Response,
) -> Response {
	let headers = resp.headers_mut();
	for (name, value) in HEADERS {
		if !headers.contains_key(name) {
			headers.insert(name, HeaderValue::from_static(value));
		}
	}

	if !headers.contains_key(header::CONTENT_SECURITY_POLICY) {
		headers.insert(header::CONTENT_SECURITY_POLICY, sec.csp.clone());
	}
	if sec.deny_frames && !headers.contains_key(header::X_FRAME_OPTIONS) {
		headers
			.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
	}

	resp
}
