//! Users log in with email and password. Admins see and manage every site,
//! other users only the sites they were given. API tokens (for scripts and
//! AI agents) act as the admin who created them, only on the sites routes.
//!
//! Session tokens, api tokens and password links are 32 random bytes, only
//! their sha256 is stored.

pub mod routes;

use std::sync::LazyLock;
use std::time::Duration;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use pg::time::{DateTime, Timeout};
use pg::{Database, FromRow, UniqueId};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tokio::sync::Semaphore;

use crate::db::{IdRow, migrate};
use crate::error::{Error, Result};

pub type Token = crypto::token::Token<32>;

const MIGRATIONS: &[(&str, &str)] = &[
	(
		"create-users",
		include_str!("./migrations/create-users.sql"),
	),
	(
		"create-password-links",
		include_str!("./migrations/create-password-links.sql"),
	),
	(
		"hash-sessions",
		include_str!("./migrations/hash-sessions.sql"),
	),
];

/// A session ends 30 days after its last use
const SESSION_TIMEOUT: Duration = Duration::from_secs(30 * 24 * 60 * 60);
/// the timeout moves on at most once an hour, most requests only read
const SESSION_EXTEND_AFTER: Duration = Duration::from_secs(60 * 60);
/// and at the latest a year after the login
const SESSION_MAX_AGE: Duration = Duration::from_secs(365 * 24 * 60 * 60);
/// 7 days
const PASSWORD_LINK_TIMEOUT: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// The password hash of a user who sets the password with a link, no
/// password matches it
pub const NO_PASSWORD: &str = "!";

/// compared against for unknown emails and users without a password, a
/// login takes as long either way
static DUMMY_HASH: LazyLock<String> =
	LazyLock::new(|| bcrypt::hash("strichli", bcrypt::DEFAULT_COST).unwrap());

/// bcrypt takes a quarter of a second of a core, at most this many run at
/// once so logins can't take all of them
static BCRYPT: Semaphore = Semaphore::const_new(2);

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct User {
	pub id: UniqueId,
	pub email: String,
	pub name: String,
	pub admin: bool,
	pub created_on: DateTime,
}

const USER_COLUMNS: &str = "id, email, name, admin, created_on";

impl User {
	pub fn require_admin(&self) -> Result<()> {
		if self.admin {
			Ok(())
		} else {
			Err(Error::AdminOnly)
		}
	}
}

#[derive(Debug, Clone, FromRow)]
pub struct Session {
	pub token_hash: String,
	pub timeout: Timeout,
	pub user_id: UniqueId,
	pub created_on: DateTime,
}

impl Session {
	fn is_valid(&self) -> bool {
		let too_old = self
			.created_on
			.abs_diff(&DateTime::now())
			.is_some_and(|age| age > SESSION_MAX_AGE);

		!self.timeout.has_elapsed() && !too_old
	}

	fn should_extend(&self) -> bool {
		self.timeout
			.remaining()
			.is_none_or(|d| d < SESSION_TIMEOUT - SESSION_EXTEND_AFTER)
	}
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ApiToken {
	pub id: UniqueId,
	pub name: String,
	pub created_on: DateTime,
	pub last_used: Option<DateTime>,
}

/// The email trimmed and lowercase, the name trimmed
pub fn normalize(email: &str, name: &str) -> Result<(String, String)> {
	let email = email.trim().to_lowercase();
	let valid = email
		.split_once('@')
		.is_some_and(|(user, host)| !user.is_empty() && host.contains('.'));
	if !valid || email.contains(char::is_whitespace) {
		return Err(Error::Invalid(format!("invalid email {email:?}")));
	}

	let name = name.trim();
	if name.is_empty() {
		return Err(Error::Invalid("the name is empty".into()));
	}

	Ok((email, name.into()))
}

/// Runs bcrypt on a blocking thread, a few at once
async fn bcrypt_blocking<T, F>(f: F) -> Result<T>
where
	F: FnOnce() -> T + Send + 'static,
	T: Send + 'static,
{
	let _permit = BCRYPT
		.acquire()
		.await
		.map_err(|e| Error::Internal(e.to_string()))?;

	tokio::task::spawn_blocking(f)
		.await
		.map_err(|e| Error::Internal(e.to_string()))
}

/// Checks the length, bcrypt only uses the first 72 bytes
pub fn check_password(password: &str) -> Result<()> {
	if password.chars().count() < 10 {
		return Err(Error::Invalid(
			"the password needs at least 10 characters".into(),
		));
	}
	if password.len() > 72 {
		return Err(Error::Invalid(
			"the password can have at most 72 bytes".into(),
		));
	}

	Ok(())
}

pub async fn hash_password(password: String) -> Result<String> {
	check_password(&password)?;

	bcrypt_blocking(move || bcrypt::hash(password, bcrypt::DEFAULT_COST))
		.await?
		.map_err(|e| Error::Internal(e.to_string()))
}

pub fn hash_token(token: &str) -> String {
	Sha256::digest(token.as_bytes())
		.iter()
		.map(|b| format!("{b:02x}"))
		.collect()
}

#[derive(Clone)]
pub struct Users {
	db: Database,
}

impl Users {
	pub async fn new(db: &Database) -> Self {
		migrate(db, MIGRATIONS).await;

		Self { db: db.clone() }
	}

	pub async fn list(&self) -> Result<Vec<User>> {
		let conn = self.db.get().await?;

		Ok(conn
			.connection()
			.query(
				&format!("SELECT {USER_COLUMNS} FROM users ORDER BY name"),
				&[],
			)
			.await?)
	}

	pub async fn by_id(&self, id: &UniqueId) -> Result<Option<User>> {
		let conn = self.db.get().await?;

		Ok(conn
			.connection()
			.query_opt(
				&format!("SELECT {USER_COLUMNS} FROM users WHERE id = $1"),
				&[id],
			)
			.await?)
	}

	pub async fn by_email(&self, email: &str) -> Result<Option<User>> {
		let email = email.trim().to_lowercase();
		let conn = self.db.get().await?;

		Ok(conn
			.connection()
			.query_opt(
				&format!("SELECT {USER_COLUMNS} FROM users WHERE email = $1"),
				&[&email],
			)
			.await?)
	}

	/// `password_hash` from `hash_password` or `NO_PASSWORD`, with the sites
	/// a user who is no admin sees
	pub async fn create(
		&self,
		email: &str,
		name: &str,
		password_hash: &str,
		admin: bool,
		site_ids: &[UniqueId],
	) -> Result<User> {
		let (email, name) = normalize(email, name)?;
		let user = User {
			id: UniqueId::new(),
			email,
			name,
			admin,
			created_on: DateTime::now(),
		};

		let mut conn = self.db.get().await?;
		let trans = conn.transaction().await?;
		let conn = trans.connection();

		conn.execute(
			"INSERT INTO users
			(id, email, name, password_hash, admin, created_on)
			VALUES ($1, $2, $3, $4, $5, $6)",
			&[
				&user.id,
				&user.email,
				&user.name,
				&password_hash,
				&user.admin,
				&user.created_on,
			],
		)
		.await?;
		set_access(&conn, &user.id, site_ids).await?;

		trans.commit().await?;
		tracing::info!("created the user {}", user.id);
		Ok(user)
	}

	/// With the sites a user who is no admin sees
	pub async fn update(
		&self,
		id: &UniqueId,
		email: &str,
		name: &str,
		admin: bool,
		site_ids: &[UniqueId],
	) -> Result<User> {
		let (email, name) = normalize(email, name)?;

		let mut conn = self.db.get().await?;
		let trans = conn.transaction().await?;
		let conn = trans.connection();

		let user: User = conn
			.query_opt(
				&format!(
					"UPDATE users SET email = $2, name = $3, admin = $4
					WHERE id = $1 RETURNING {USER_COLUMNS}"
				),
				&[id, &email, &name, &admin],
			)
			.await?
			.ok_or(Error::NotFound)?;
		set_access(&conn, id, site_ids).await?;

		trans.commit().await?;
		Ok(user)
	}

	/// `password_hash` from `hash_password`, ends every other session and
	/// the password links
	pub async fn set_password(
		&self,
		id: &UniqueId,
		password_hash: &str,
		keep_session: Option<&str>,
	) -> Result<()> {
		let mut conn = self.db.get().await?;
		let trans = conn.transaction().await?;
		let conn = trans.connection();

		conn.execute(
			"UPDATE users SET password_hash = $2 WHERE id = $1",
			&[id, &password_hash],
		)
		.await?;
		conn.execute(
			"DELETE FROM user_sessions
			WHERE user_id = $1 AND token_hash IS DISTINCT FROM $2",
			&[id, &keep_session],
		)
		.await?;
		conn.execute("DELETE FROM password_links WHERE user_id = $1", &[id])
			.await?;

		trans.commit().await?;
		Ok(())
	}

	/// Sessions, tokens and site access go with it via the foreign keys
	pub async fn delete(&self, id: &UniqueId) -> Result<()> {
		let conn = self.db.get().await?;

		conn.connection()
			.execute("DELETE FROM users WHERE id = $1", &[id])
			.await?;

		Ok(())
	}

	/// The user if the email and password are right. An unknown email or a
	/// user without a password takes as long as a wrong password.
	pub async fn check_login(
		&self,
		email: &str,
		password: String,
	) -> Result<Option<User>> {
		let email = email.trim().to_lowercase();
		let conn = self.db.get().await?;

		let found: Option<LoginRow> = conn
			.connection()
			.query_opt(
				"SELECT id, password_hash FROM users WHERE email = $1",
				&[&email],
			)
			.await?;
		drop(conn);

		let hash = found
			.as_ref()
			.map(|f| f.password_hash.clone())
			.filter(|h| h != NO_PASSWORD)
			.unwrap_or_else(|| DUMMY_HASH.clone());
		let right = bcrypt_blocking(move || {
			bcrypt::verify(&password, &hash).unwrap_or(false)
		})
		.await?;

		match found.filter(|f| right && f.password_hash != NO_PASSWORD) {
			Some(f) => self.by_id(&f.id).await,
			None => Ok(None),
		}
	}

	/// Returns the token of the new session
	pub async fn new_session(&self, user_id: &UniqueId) -> Result<String> {
		let token = Token::new().to_string();
		let conn = self.db.get().await?;

		conn.connection()
			.execute(
				"INSERT INTO user_sessions
				(token_hash, timeout, user_id, created_on)
				VALUES ($1, $2, $3, $4)",
				&[
					&hash_token(&token),
					&Timeout::new(SESSION_TIMEOUT),
					user_id,
					&DateTime::now(),
				],
			)
			.await?;

		Ok(token)
	}

	/// The session of a token if it is valid, its timeout moves on
	pub async fn session(&self, token: &str) -> Result<Option<Session>> {
		let conn = self.db.get().await?;
		let conn = conn.connection();

		let session: Option<Session> = conn
			.query_opt(
				"SELECT token_hash, timeout, user_id, created_on
				FROM user_sessions WHERE token_hash = $1",
				&[&hash_token(token)],
			)
			.await?;
		let Some(mut session) = session.filter(Session::is_valid) else {
			return Ok(None);
		};

		if session.should_extend() {
			session.timeout = Timeout::new(SESSION_TIMEOUT);
			conn.execute(
				"UPDATE user_sessions SET timeout = $2 WHERE token_hash = $1",
				&[&session.token_hash, &session.timeout],
			)
			.await?;
		}

		Ok(Some(session))
	}

	pub async fn delete_session(&self, token_hash: &str) -> Result<()> {
		let conn = self.db.get().await?;

		conn.connection()
			.execute(
				"DELETE FROM user_sessions WHERE token_hash = $1",
				&[&token_hash],
			)
			.await?;

		Ok(())
	}

	pub async fn tokens(&self, user_id: &UniqueId) -> Result<Vec<ApiToken>> {
		let conn = self.db.get().await?;

		Ok(conn
			.connection()
			.query(
				"SELECT id, name, created_on, last_used FROM api_tokens
				WHERE user_id = $1 ORDER BY created_on",
				&[user_id],
			)
			.await?)
	}

	/// Returns the token, only its hash is stored
	pub async fn create_token(
		&self,
		user_id: &UniqueId,
		name: &str,
	) -> Result<(ApiToken, String)> {
		let name = name.trim();
		if name.is_empty() {
			return Err(Error::Invalid("the name is empty".into()));
		}

		let secret = format!("stri_{}", Token::new());
		let token = ApiToken {
			id: UniqueId::new(),
			name: name.into(),
			created_on: DateTime::now(),
			last_used: None,
		};

		let conn = self.db.get().await?;
		conn.connection()
			.execute(
				"INSERT INTO api_tokens
				(id, name, token_hash, user_id, created_on)
				VALUES ($1, $2, $3, $4, $5)",
				&[
					&token.id,
					&token.name,
					&hash_token(&secret),
					user_id,
					&token.created_on,
				],
			)
			.await?;

		Ok((token, secret))
	}

	pub async fn delete_token(
		&self,
		user_id: &UniqueId,
		id: &UniqueId,
	) -> Result<()> {
		let conn = self.db.get().await?;

		let n = conn
			.connection()
			.execute(
				"DELETE FROM api_tokens WHERE id = $1 AND user_id = $2",
				&[id, user_id],
			)
			.await?;

		if n == 0 {
			return Err(Error::NotFound);
		}
		Ok(())
	}

	/// The user of an api token, marks the token as used
	pub async fn by_token(&self, secret: &str) -> Result<Option<User>> {
		let conn = self.db.get().await?;

		let row: Option<IdRow> = conn
			.connection()
			.query_opt(
				"UPDATE api_tokens SET last_used = $2
				WHERE token_hash = $1 RETURNING user_id AS id",
				&[&hash_token(secret), &DateTime::now()],
			)
			.await?;
		drop(conn);

		match row {
			Some(r) => self.by_id(&r.id).await,
			None => Ok(None),
		}
	}

	/// A link to set the password, replaces the user's earlier links.
	/// Returns the token, only its hash is stored.
	pub async fn create_password_link(
		&self,
		user_id: &UniqueId,
	) -> Result<String> {
		let token = Token::new().to_string();
		let mut conn = self.db.get().await?;
		let trans = conn.transaction().await?;
		let conn = trans.connection();

		conn.execute(
			"DELETE FROM password_links WHERE user_id = $1",
			&[user_id],
		)
		.await?;
		conn.execute(
			"INSERT INTO password_links (token_hash, user_id, timeout)
			VALUES ($1, $2, $3)",
			&[
				&hash_token(&token),
				user_id,
				&Timeout::new(PASSWORD_LINK_TIMEOUT),
			],
		)
		.await?;

		trans.commit().await?;
		Ok(token)
	}

	/// The user of a password link that is not expired
	pub async fn password_link_user(
		&self,
		token: &str,
	) -> Result<Option<User>> {
		let conn = self.db.get().await?;

		let link: Option<LinkRow> = conn
			.connection()
			.query_opt(
				"SELECT user_id, timeout FROM password_links
				WHERE token_hash = $1",
				&[&hash_token(token)],
			)
			.await?;
		drop(conn);

		match link.filter(|l| !l.timeout.has_elapsed()) {
			Some(l) => self.by_id(&l.user_id).await,
			None => Ok(None),
		}
	}

	/// Uses the link up and sets the password, a link works only once even
	/// if it is sent twice at the same time
	pub async fn use_password_link(
		&self,
		token: &str,
		password_hash: &str,
	) -> Result<Option<User>> {
		let conn = self.db.get().await?;

		let link: Option<LinkRow> = conn
			.connection()
			.query_opt(
				"DELETE FROM password_links WHERE token_hash = $1
				RETURNING user_id, timeout",
				&[&hash_token(token)],
			)
			.await?;
		drop(conn);

		let Some(link) = link.filter(|l| !l.timeout.has_elapsed()) else {
			return Ok(None);
		};
		self.set_password(&link.user_id, password_hash, None)
			.await?;

		self.by_id(&link.user_id).await
	}
}

/// Replaces the sites a user was given (the table belongs to the sites)
async fn set_access(
	conn: &pg::Connection<'_>,
	user_id: &UniqueId,
	site_ids: &[UniqueId],
) -> Result<()> {
	conn.execute("DELETE FROM site_users WHERE user_id = $1", &[user_id])
		.await?;
	for site_id in site_ids {
		conn.execute(
			"INSERT INTO site_users (site_id, user_id)
			SELECT id, $2 FROM sites WHERE id = $1",
			&[site_id, user_id],
		)
		.await?;
	}

	Ok(())
}

#[derive(FromRow)]
struct LinkRow {
	user_id: UniqueId,
	timeout: Timeout,
}

#[derive(FromRow)]
struct LoginRow {
	id: UniqueId,
	password_hash: String,
}

fn session_token(parts: &Parts) -> Option<&str> {
	parts.headers.get("session-token")?.to_str().ok()
}

fn bearer(parts: &Parts) -> Option<&str> {
	parts
		.headers
		.get("authorization")?
		.to_str()
		.ok()?
		.strip_prefix("Bearer ")
		.map(str::trim)
}

/// The user of a valid session from the `session-token` header, only the
/// webui
pub struct Authed {
	pub session: Session,
	pub user: User,
}

impl<S> FromRequestParts<S> for Authed
where
	S: Send + Sync,
	Users: FromRef<S>,
{
	type Rejection = Error;

	async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
		let token = session_token(parts).ok_or(Error::InvalidSessionToken)?;
		let users = Users::from_ref(state);

		let session = users
			.session(token)
			.await?
			.ok_or(Error::InvalidSessionToken)?;
		let user = users
			.by_id(&session.user_id)
			.await?
			.ok_or(Error::InvalidSessionToken)?;

		Ok(Self { session, user })
	}
}

/// The user of a session or of an api token
/// (`authorization: Bearer <token>`)
pub struct Caller {
	pub user: User,
	pub via_token: bool,
}

impl<S> FromRequestParts<S> for Caller
where
	S: Send + Sync,
	Users: FromRef<S>,
{
	type Rejection = Error;

	async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
		if let Some(secret) = bearer(parts) {
			let user = Users::from_ref(state)
				.by_token(secret)
				.await?
				.ok_or(Error::InvalidApiToken)?;

			return Ok(Self {
				user,
				via_token: true,
			});
		}

		let authed = Authed::from_request_parts(parts, state).await?;
		Ok(Self {
			user: authed.user,
			via_token: false,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn normalizes_email_and_name() {
		let (email, name) =
			normalize(" Soeren@Meierlabs.ch ", " Sören ").unwrap();
		assert_eq!(email, "soeren@meierlabs.ch");
		assert_eq!(name, "Sören");

		assert!(normalize("no-at", "x").is_err());
		assert!(normalize("a@b", "x").is_err());
		assert!(normalize("a@b.ch", " ").is_err());
	}

	#[test]
	fn token_hash_is_hex_sha256() {
		let hash = hash_token("abc");
		assert_eq!(hash.len(), 64);
		assert!(hash.starts_with("ba7816bf"));
	}

	#[test]
	fn password_lengths() {
		assert!(check_password("short").is_err());
		assert!(check_password("long enough").is_ok());
		assert!(check_password(&"x".repeat(72)).is_ok());
		assert!(check_password(&"x".repeat(73)).is_err());
	}

	#[test]
	fn no_password_never_matches() {
		assert!(!bcrypt::verify("!", NO_PASSWORD).unwrap_or(false));
	}

	#[test]
	fn sessions_extend_and_end() {
		let session = |timeout: Duration, age: Duration| Session {
			token_hash: String::new(),
			timeout: Timeout::new(timeout),
			user_id: UniqueId::new(),
			created_on: DateTime::from_std(std::time::SystemTime::now() - age),
		};
		let day = Duration::from_secs(24 * 60 * 60);

		let fresh = session(SESSION_TIMEOUT, Duration::ZERO);
		assert!(fresh.is_valid());
		assert!(!fresh.should_extend());

		let used_yesterday = session(SESSION_TIMEOUT - day, day);
		assert!(used_yesterday.should_extend());

		let too_old = session(SESSION_TIMEOUT, SESSION_MAX_AGE + day);
		assert!(!too_old.is_valid());
	}
}
