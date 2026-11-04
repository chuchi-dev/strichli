//! Sites and their stats, for the webui and api tokens

use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use pg::UniqueId;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::error::{Error, Result};
use crate::events::{Bucket, Range, Stats, ValueCount};
use crate::users::Caller;

use super::Site;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SiteResp {
	#[serde(flatten)]
	site: Site,
	/// the script tag for the site's pages
	snippet: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SiteOverview {
	#[serde(flatten)]
	site: SiteResp,
	/// visitors and pageviews of the last 30 days, today last
	days: Vec<Bucket>,
}

fn site_resp(state: &AppState, site: Site) -> SiteResp {
	SiteResp {
		snippet: format!(
			r#"<script async src="{}s.js"></script>"#,
			state.cfg.public_addr
		),
		site,
	}
}

async fn list(
	State(state): State<AppState>,
	caller: Caller,
) -> Result<Json<Vec<SiteOverview>>> {
	let sites = state.sites.list(&caller.user).await?;
	let ids: Vec<UniqueId> = sites.iter().map(|s| s.id).collect();
	let mut days = state.events.last_30_days(&ids).await?;

	let overview = sites
		.into_iter()
		.map(|site| SiteOverview {
			days: days.remove(&site.id.to_string()).unwrap_or_default(),
			site: site_resp(&state, site),
		})
		.collect();

	Ok(Json(overview))
}

#[derive(Debug, Deserialize)]
struct SiteReq {
	domain: String,
	name: String,
}

async fn create(
	State(state): State<AppState>,
	caller: Caller,
	Json(req): Json<SiteReq>,
) -> Result<Json<SiteResp>> {
	caller.user.require_admin()?;

	let site = state.sites.create(&req.domain, &req.name).await?;
	Ok(Json(site_resp(&state, site)))
}

async fn get_site(
	State(state): State<AppState>,
	caller: Caller,
	Path(id): Path<UniqueId>,
) -> Result<Json<SiteResp>> {
	let site = state.sites.get(&caller.user, &id).await?;
	Ok(Json(site_resp(&state, site)))
}

async fn update(
	State(state): State<AppState>,
	caller: Caller,
	Path(id): Path<UniqueId>,
	Json(req): Json<SiteReq>,
) -> Result<Json<SiteResp>> {
	caller.user.require_admin()?;

	let site = state.sites.update(&id, &req.domain, &req.name).await?;
	Ok(Json(site_resp(&state, site)))
}

/// Deletes all its events too, so not with an api token
async fn delete(
	State(state): State<AppState>,
	caller: Caller,
	Path(id): Path<UniqueId>,
) -> Result<Json<()>> {
	caller.user.require_admin()?;
	if caller.via_token {
		return Err(Error::Invalid(
			"sites can only be deleted in the webui".into(),
		));
	}

	state.sites.delete(&id).await?;
	Ok(Json(()))
}

async fn stats(
	State(state): State<AppState>,
	caller: Caller,
	Path(id): Path<UniqueId>,
	Query(range): Query<Range>,
) -> Result<Json<Stats>> {
	let site = state.sites.get(&caller.user, &id).await?;

	Ok(Json(state.events.stats(&site.id, &range).await?))
}

async fn values(
	State(state): State<AppState>,
	caller: Caller,
	Path((id, key)): Path<(UniqueId, String)>,
	Query(range): Query<Range>,
) -> Result<Json<Vec<ValueCount>>> {
	let site = state.sites.get(&caller.user, &id).await?;

	Ok(Json(state.events.values(&site.id, &range, &key).await?))
}

pub fn routes() -> Router<AppState> {
	Router::new()
		.route("/", get(list).post(create))
		.route("/{id}", get(get_site).put(update).delete(delete))
		.route("/{id}/stats", get(stats))
		.route("/{id}/keys/{key}", get(values))
}
