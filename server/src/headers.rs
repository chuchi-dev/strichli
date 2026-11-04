//! Security headers for the webui

use axum::http::{HeaderName, HeaderValue, header};
use axum::response::Response;

const CSP: &str = "\
default-src 'self'; \
script-src 'self'; \
style-src 'self' 'unsafe-inline'; \
img-src 'self' data:; \
connect-src 'self'; \
frame-ancestors 'none'";

const HEADERS: &[(HeaderName, &str)] = &[
	(header::STRICT_TRANSPORT_SECURITY, "max-age=31536000"),
	(header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
	(header::X_FRAME_OPTIONS, "DENY"),
	(header::REFERRER_POLICY, "same-origin"),
	(header::CONTENT_SECURITY_POLICY, CSP),
];

/// Use with `axum::middleware::map_response`
pub async fn security_headers(mut resp: Response) -> Response {
	let headers = resp.headers_mut();
	for (name, value) in HEADERS {
		if !headers.contains_key(name) {
			headers.insert(name, HeaderValue::from_static(value));
		}
	}

	resp
}
