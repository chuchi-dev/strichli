use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use pg::UniqueId;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::error::{Error, Result};
use crate::limits::ClientIp;

use super::{
	ApiToken, Authed, NO_PASSWORD, User, Users, check_password, hash_password,
};

/// The page that sets the password with a link's token
pub fn password_url(public_addr: &str, token: &str) -> String {
	format!("{public_addr}password?token={token}")
}

#[derive(Debug, Serialize)]
struct LoginResp {
	token: String,
	user: User,
}

#[derive(Debug, Deserialize)]
struct LoginReq {
	email: String,
	password: String,
}

/// Limited per client and, against many clients, per email
async fn login(
	State(state): State<AppState>,
	ip: ClientIp,
	Json(req): Json<LoginReq>,
) -> Result<Json<LoginResp>> {
	state.login_limits.check(ip)?;
	let email = req.email.trim().to_lowercase();
	state.login_failures.check(&email)?;

	let Some(user) = state.users.check_login(&email, req.password).await?
	else {
		state.login_failures.failed(&email);
		// the email stays out of the log, it might be a password
		tracing::info!("failed login");
		// slow down guessing
		tokio::time::sleep(Duration::from_secs(1)).await;
		return Err(Error::LoginIncorrect);
	};

	state.login_failures.clear(&email);
	let token = state.users.new_session(&user.id).await?;
	Ok(Json(LoginResp { token, user }))
}

async fn me(authed: Authed) -> Json<User> {
	Json(authed.user)
}

async fn logout(
	State(users): State<Users>,
	authed: Authed,
) -> Result<Json<()>> {
	users.delete_session(&authed.session.token_hash).await?;

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
	State(state): State<AppState>,
	authed: Authed,
	Json(req): Json<PasswordReq>,
) -> Result<Json<()>> {
	check_password(&req.new_password)?;
	let email = &authed.user.email;
	state.login_failures.check(email)?;

	if state
		.users
		.check_login(email, req.password)
		.await?
		.is_none()
	{
		state.login_failures.failed(email);
		return Err(Error::Invalid("the current password is wrong".into()));
	}

	let hash = hash_password(req.new_password).await?;
	state
		.users
		.set_password(&authed.user.id, &hash, Some(&authed.session.token_hash))
		.await?;

	Ok(Json(()))
}

#[derive(Debug, Deserialize)]
struct LinkQuery {
	token: String,
}

#[derive(Debug, Serialize)]
struct LinkUser {
	email: String,
	name: String,
}

/// Who a password link is for
async fn password_link(
	State(state): State<AppState>,
	ip: ClientIp,
	Query(q): Query<LinkQuery>,
) -> Result<Json<LinkUser>> {
	// the token can't be guessed, the page loads it right before the submit
	state.collect_limits.check(ip)?;

	let user = state
		.users
		.password_link_user(&q.token)
		.await?
		.ok_or(Error::NotFound)?;

	Ok(Json(LinkUser {
		email: user.email,
		name: user.name,
	}))
}

#[derive(Debug, Deserialize)]
struct SetPasswordReq {
	token: String,
	password: String,
}

/// Sets the password with a link and logs in
async fn set_password(
	State(state): State<AppState>,
	ip: ClientIp,
	Json(req): Json<SetPasswordReq>,
) -> Result<Json<LoginResp>> {
	state.login_limits.check(ip)?;
	check_password(&req.password)?;

	// no bcrypt for links that don't work
	if state.users.password_link_user(&req.token).await?.is_none() {
		return Err(Error::NotFound);
	}
	let hash = hash_password(req.password).await?;
	let user = state
		.users
		.use_password_link(&req.token, &hash)
		.await?
		.ok_or(Error::NotFound)?;
	let token = state.users.new_session(&user.id).await?;

	Ok(Json(LoginResp { token, user }))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserResp {
	#[serde(flatten)]
	user: User,
	/// the sites a user who is no admin sees
	site_ids: Vec<UniqueId>,
	/// for a new user without a password: where they set it, valid 7 days
	#[serde(skip_serializing_if = "Option::is_none")]
	password_link: Option<String>,
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
			password_link: None,
		})
		.collect();

	Ok(Json(users))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserReq {
	email: String,
	name: String,
	/// a new user without one sets it with a link, else only to change it
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

	let hash = match req.password.filter(|p| !p.is_empty()) {
		Some(p) => Some(hash_password(p).await?),
		None => None,
	};
	let user = state
		.users
		.create(
			&req.email,
			&req.name,
			hash.as_deref().unwrap_or(NO_PASSWORD),
			req.admin,
			&req.site_ids,
		)
		.await?;

	let password_link = match hash {
		Some(_) => None,
		None => {
			let token = state.users.create_password_link(&user.id).await?;
			Some(password_url(&state.cfg.public_addr, &token))
		}
	};

	Ok(Json(UserResp {
		user,
		site_ids: req.site_ids,
		password_link,
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
		.update(&id, &req.email, &req.name, req.admin, &req.site_ids)
		.await?;
	if let Some(hash) = hash {
		let keep = (id == authed.user.id)
			.then_some(authed.session.token_hash.as_str());
		state.users.set_password(&id, &hash, keep).await?;
	}

	Ok(Json(UserResp {
		user,
		site_ids: req.site_ids,
		password_link: None,
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
		.route("/login", post(login))
		.route("/me", get(me))
		.route("/logout", post(logout))
		.route("/password", post(change_password))
		.route("/password-link", get(password_link).post(set_password))
		.route("/", get(list).post(create))
		.route("/{id}", put(update).delete(delete_user))
}

pub fn token_routes() -> Router<AppState> {
	Router::new()
		.route("/", get(list_tokens).post(create_token))
		.route("/{id}", delete(delete_token))
}
