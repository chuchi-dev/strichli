use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::AppState;
use crate::error::{Error, Result};
use crate::limits::ClientIp;
use crate::sites::normalize_domain;

use super::{Payload, is_bot};

const SCRIPT: &str = include_str!("./s.js");

/// The tracking script, browsers cache it for an hour
async fn script() -> impl IntoResponse {
	(
		[
			(header::CONTENT_TYPE, "text/javascript; charset=utf-8"),
			(header::CACHE_CONTROL, "public, max-age=3600"),
		],
		SCRIPT,
	)
}

/// A pageview or key, sent as text/plain by `sendBeacon` so browsers don't
/// need a preflight. The ip only counts against the rate limit.
async fn collect(
	State(state): State<AppState>,
	ip: ClientIp,
	headers: HeaderMap,
	body: Bytes,
) -> Result<StatusCode> {
	state.collect_limits.check(ip)?;

	let header = |name| headers.get(name).and_then(|v| v.to_str().ok());
	if is_bot(header(header::USER_AGENT)) {
		return Ok(StatusCode::NO_CONTENT);
	}

	let payload: Payload = serde_json::from_slice(&body)
		.map_err(|e| Error::Invalid(format!("invalid event: {e}")))?;
	let domain = normalize_domain(&payload.h)?;

	// a browser sends the page's origin, it must be the same site
	if let Some(origin) = header(header::ORIGIN)
		&& normalize_domain(origin).ok().as_deref() != Some(&domain)
	{
		return Err(Error::Invalid("the origin is not the host".into()));
	}

	let site_id = state.sites.by_host(&domain).await?.ok_or(Error::NotFound)?;
	let event = payload.into_event(&domain)?;
	state.events.insert(&site_id, &event).await?;

	Ok(StatusCode::NO_CONTENT)
}

async fn version() -> Json<serde_json::Value> {
	Json(serde_json::json!({ "version": env!("CARGO_PKG_VERSION") }))
}

pub fn routes() -> Router<AppState> {
	Router::new()
		.route("/s.js", get(script))
		.route("/e", post(collect).layer(DefaultBodyLimit::max(4096)))
		.route("/api/version", get(version))
}
