//! Serves the built webui: static files as they are, every other path gets
//! the index.html and the router of the webui takes over.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use tower::ServiceExt as _;
use tower_http::services::ServeDir;

use crate::AppState;

#[derive(Clone)]
pub struct Webui {
	dist_dir: PathBuf,
	index: Arc<Option<String>>,
}

impl Webui {
	pub fn new(dist_dir: &str) -> Self {
		let dir = Path::new(dist_dir);
		let index = std::fs::read_to_string(dir.join("index.html")).ok();
		if index.is_none() {
			tracing::warn!("no webui found in {dist_dir}");
		}

		Self {
			dist_dir: dir.into(),
			index: Arc::new(index),
		}
	}
}

async fn index(State(webui): State<Webui>, req: Request<Body>) -> Response {
	let Some(index) = webui.index.as_ref() else {
		return StatusCode::NOT_FOUND.into_response();
	};

	let path = req.uri().path();
	if path.starts_with("/api/") {
		return StatusCode::NOT_FOUND.into_response();
	}

	if path != "/" && path != "/index.html" {
		let resp = ServeDir::new(&webui.dist_dir)
			.append_index_html_on_directories(false)
			.oneshot(req)
			.await
			.unwrap();
		if resp.status() != StatusCode::NOT_FOUND {
			return resp.map(Body::new);
		}
	}

	([(header::CACHE_CONTROL, "no-cache")], Html(index.clone())).into_response()
}

pub fn routes() -> Router<AppState> {
	Router::new()
		.route("/", get(index))
		.route("/{*rest}", get(index))
}
