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

use super::{Payload, is_bot, noscript_pageview};

const SCRIPT: &str = include_str!("./s.js");

/// a transparent 1x1 gif
const GIF: &[u8] = &[
	0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00,
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0xf9, 0x04, 0x01, 0x00,
	0x00, 0x00, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00,
	0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3b,
];

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

/// The image in `<noscript>`: a pageview of the page in the referer. It
/// always returns the image, a failure only shows in the log.
async fn noscript(
	State(state): State<AppState>,
	ip: ClientIp,
	headers: HeaderMap,
) -> impl IntoResponse {
	if let Err(e) = noscript_inner(&state, ip, &headers).await {
		tracing::debug!("noscript pageview not counted: {e}");
	}

	(
		[
			(header::CONTENT_TYPE, "image/gif"),
			(header::CACHE_CONTROL, "no-store"),
		],
		GIF,
	)
}

async fn noscript_inner(
	state: &AppState,
	ip: ClientIp,
	headers: &HeaderMap,
) -> Result<()> {
	state.collect_limits.check(ip)?;

	let header = |name| headers.get(name).and_then(|v| v.to_str().ok());
	if is_bot(header(header::USER_AGENT)) {
		return Ok(());
	}

	let page = header(header::REFERER)
		.ok_or_else(|| Error::Invalid("no referer".into()))?;
	let (domain, event) = noscript_pageview(page)?;
	let site_id = state.sites.by_host(&domain).await?.ok_or(Error::NotFound)?;

	state.events.insert(&site_id, &event).await
}

async fn version() -> Json<serde_json::Value> {
	Json(serde_json::json!({ "version": env!("CARGO_PKG_VERSION") }))
}

pub fn routes() -> Router<AppState> {
	Router::new()
		.route("/s.js", get(script))
		// the paths of simple analytics, sa.goodserver.ch can point here
		.route("/latest.js", get(script))
		.route("/noscript.gif", get(noscript))
		.route("/e", post(collect).layer(DefaultBodyLimit::max(4096)))
		.route("/api/version", get(version))
}
