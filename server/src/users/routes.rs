use std::time::Duration;

use axum::extract::{Path, State};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use pg::UniqueId;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::error::{Error, Result};
use crate::limits::ClientIp;

use super::{
	ApiToken, Authed, AuthedSession, Session, User, Users, hash_password,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Authenticated {
	session: Session,
	user: User,
}

#[derive(Debug, Deserialize)]
struct LoginReq {
	email: String,
	password: String,
}

async fn login(
	State(state): State<AppState>,
	ip: ClientIp,
	Json(req): Json<LoginReq>,
) -> Result<Json<Authenticated>> {
	state.login_limits.check(ip)?;

	let Some(user) = state.users.check_login(&req.email, req.password).await?
	else {
		tracing::info!("failed login for {}", req.email.trim());
		// slow down guessing
		tokio::time::sleep(Duration::from_secs(1)).await;
		return Err(Error::LoginIncorrect);
	};

	let session = state.users.new_session(&user.id).await?;
	Ok(Json(Authenticated { session, user }))
}

async fn token_auth(
	State(users): State<Users>,
	authed: Authed,
) -> Result<Json<Authenticated>> {
	let mut session = authed.session;
	if session.should_renew() {
		session = users.renew_session(session).await?;
	}

	Ok(Json(Authenticated {
		session,
		user: authed.user,
	}))
}

async fn logout(
	State(users): State<Users>,
	AuthedSession(session): AuthedSession,
) -> Result<Json<()>> {
	users.delete_session(&session.token).await?;

	Ok(Json(()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PasswordReq {
	password: String,
	new_password: String,
}

/// Changes the own password, ends the other sessions
async fn change_password(
	State(users): State<Users>,
	authed: Authed,
	Json(req): Json<PasswordReq>,
) -> Result<Json<()>> {
	if users
		.check_login(&authed.user.email, req.password)
		.await?
		.is_none()
	{
		return Err(Error::Invalid("the current password is wrong".into()));
	}

	let hash = hash_password(req.new_password).await?;
	users
		.set_password(&authed.user.id, &hash, Some(&authed.session.token))
		.await?;

	Ok(Json(()))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserResp {
	#[serde(flatten)]
	user: User,
	/// the sites a user who is no admin sees
	site_ids: Vec<UniqueId>,
}

async fn list(
	State(state): State<AppState>,
	authed: Authed,
) -> Result<Json<Vec<UserResp>>> {
	authed.user.require_admin()?;

	let mut access = state.sites.access().await?;
	let users = state
		.users
		.list()
		.await?
		.into_iter()
		.map(|user| UserResp {
			site_ids: access.remove(&user.id).unwrap_or_default(),
			user,
		})
		.collect();

	Ok(Json(users))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserReq {
	email: String,
	name: String,
	/// required for a new user, else only set to change it
	password: Option<String>,
	admin: bool,
	#[serde(default)]
	site_ids: Vec<UniqueId>,
}

async fn create(
	State(state): State<AppState>,
	authed: Authed,
	Json(req): Json<UserReq>,
) -> Result<Json<UserResp>> {
	authed.user.require_admin()?;

	let password = req
		.password
		.ok_or_else(|| Error::Invalid("the password is missing".into()))?;
	let hash = hash_password(password).await?;
	let user = state
		.users
		.create(&req.email, &req.name, &hash, req.admin)
		.await?;
	state.sites.set_access(&user.id, &req.site_ids).await?;

	Ok(Json(UserResp {
		user,
		site_ids: req.site_ids,
	}))
}

async fn update(
	State(state): State<AppState>,
	authed: Authed,
	Path(id): Path<UniqueId>,
	Json(req): Json<UserReq>,
) -> Result<Json<UserResp>> {
	authed.user.require_admin()?;
	if id == authed.user.id && !req.admin {
		return Err(Error::Invalid("you can't remove your own admin".into()));
	}

	let hash = match req.password.filter(|p| !p.is_empty()) {
		Some(p) => Some(hash_password(p).await?),
		None => None,
	};
	let user = state
		.users
		.update(&id, &req.email, &req.name, req.admin)
		.await?;
	if let Some(hash) = hash {
		let keep = (id == authed.user.id).then_some(&authed.session.token);
		state.users.set_password(&id, &hash, keep).await?;
	}
	state.sites.set_access(&id, &req.site_ids).await?;

	Ok(Json(UserResp {
		user,
		site_ids: req.site_ids,
	}))
}

async fn delete_user(
	State(users): State<Users>,
	authed: Authed,
	Path(id): Path<UniqueId>,
) -> Result<Json<()>> {
	authed.user.require_admin()?;
	if id == authed.user.id {
		return Err(Error::Invalid("you can't delete yourself".into()));
	}

	users.delete(&id).await?;
	Ok(Json(()))
}

/// The own api tokens, only admins have them
async fn list_tokens(
	State(users): State<Users>,
	authed: Authed,
) -> Result<Json<Vec<ApiToken>>> {
	authed.user.require_admin()?;

	Ok(Json(users.tokens(&authed.user.id).await?))
}

#[derive(Debug, Deserialize)]
struct TokenReq {
	name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NewToken {
	#[serde(flatten)]
	token: ApiToken,
	/// only shown this once
	secret: String,
}

async fn create_token(
	State(users): State<Users>,
	authed: Authed,
	Json(req): Json<TokenReq>,
) -> Result<Json<NewToken>> {
	authed.user.require_admin()?;

	let (token, secret) =
		users.create_token(&authed.user.id, &req.name).await?;
	Ok(Json(NewToken { token, secret }))
}

async fn delete_token(
	State(users): State<Users>,
	authed: Authed,
	Path(id): Path<UniqueId>,
) -> Result<Json<()>> {
	users.delete_token(&authed.user.id, &id).await?;

	Ok(Json(()))
}

pub fn routes() -> Router<AppState> {
	Router::new()
		.route("/users/login", post(login))
		.route("/users/tokenauth", post(token_auth))
		.route("/users/logout", post(logout))
		.route("/users/password", post(change_password))
		.route("/users", get(list).post(create))
		.route("/users/{id}", put(update).delete(delete_user))
		.route("/tokens", get(list_tokens).post(create_token))
		.route("/tokens/{id}", delete(delete_token))
}
