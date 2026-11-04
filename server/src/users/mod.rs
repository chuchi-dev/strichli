//! Users log in with email and password. Admins see and manage every site,
//! other users only the sites they were given. API tokens (for scripts and
//! AI agents) act as the admin who created them, only on the sites routes.

pub mod routes;

use std::sync::LazyLock;
use std::time::Duration;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use pg::time::{DateTime, Timeout};
use pg::{Database, FromRow, UniqueId};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

pub type Token = crypto::token::Token<32>;

const MIGRATIONS: &[(&str, &str)] = &[(
	"create-users",
	include_str!("./migrations/create-users.sql"),
)];

/// 30 days
const SESSION_TIMEOUT: Duration = Duration::from_secs(30 * 24 * 60 * 60);
/// renew the session once less than 10 days are remaining
const RENEW_AFTER: Duration = Duration::from_secs(10 * 24 * 60 * 60);

/// compared against for unknown emails, a login takes as long either way
static DUMMY_HASH: LazyLock<String> =
	LazyLock::new(|| bcrypt::hash("strichli", bcrypt::DEFAULT_COST).unwrap());

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

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Session {
	pub token: Token,
	pub timeout: Timeout,
	#[serde(skip)]
	pub user_id: UniqueId,
}

impl Session {
	fn should_renew(&self) -> bool {
		self.timeout
			.remaining()
			.map(|d| d < RENEW_AFTER)
			.unwrap_or(true)
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

/// bcrypt takes a quarter of a second, so it runs on a blocking thread
pub async fn hash_password(password: String) -> Result<String> {
	if password.chars().count() < 10 {
		return Err(Error::Invalid(
			"the password needs at least 10 characters".into(),
		));
	}

	tokio::task::spawn_blocking(move || {
		bcrypt::hash(password, bcrypt::DEFAULT_COST)
	})
	.await
	.map_err(|e| Error::Internal(e.to_string()))?
	.map_err(|e| Error::Internal(e.to_string()))
}

fn hash_token(token: &str) -> String {
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
		let migrations = db.migrations();
		let mut conn = db.get().await.unwrap();

		for (name, sql) in MIGRATIONS {
			migrations
				.add(&mut conn, name, sql)
				.await
				.expect("failed to run migration");
		}

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

	/// `password_hash` from `hash_password`
	pub async fn create(
		&self,
		email: &str,
		name: &str,
		password_hash: &str,
		admin: bool,
	) -> Result<User> {
		let (email, name) = normalize(email, name)?;
		let conn = self.db.get().await?;
		let conn = conn.connection();

		let user = User {
			id: UniqueId::new(),
			email,
			name,
			admin,
			created_on: DateTime::now(),
		};
		let inserted = conn
			.execute(
				"INSERT INTO users
				(id, email, name, password_hash, admin, created_on)
				VALUES ($1, $2, $3, $4, $5, $6)
				ON CONFLICT (email) DO NOTHING",
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
		if inserted == 0 {
			return Err(Error::Conflict(format!(
				"{} already has an account",
				user.email
			)));
		}

		tracing::info!("created the user {} {}", user.email, user.id);
		Ok(user)
	}

	pub async fn update(
		&self,
		id: &UniqueId,
		email: &str,
		name: &str,
		admin: bool,
	) -> Result<User> {
		let (email, name) = normalize(email, name)?;
		let conn = self.db.get().await?;

		let taken: Option<UniqueId> = conn
			.connection()
			.query_opt::<IdRow, _>(
				"SELECT id FROM users WHERE email = $1 AND id <> $2",
				&[&email, id],
			)
			.await?
			.map(|r| r.id);
		if taken.is_some() {
			return Err(Error::Conflict(format!(
				"{email} already has an account"
			)));
		}

		conn.connection()
			.query_opt(
				&format!(
					"UPDATE users SET email = $2, name = $3, admin = $4
					WHERE id = $1 RETURNING {USER_COLUMNS}"
				),
				&[id, &email, &name, &admin],
			)
			.await?
			.ok_or(Error::NotFound)
	}

	/// `password_hash` from `hash_password`, ends every other session
	pub async fn set_password(
		&self,
		id: &UniqueId,
		password_hash: &str,
		keep_session: Option<&Token>,
	) -> Result<()> {
		let conn = self.db.get().await?;
		let conn = conn.connection();

		conn.execute(
			"UPDATE users SET password_hash = $2 WHERE id = $1",
			&[id, &password_hash],
		)
		.await?;
		conn.execute(
			"DELETE FROM user_sessions
			WHERE user_id = $1 AND token IS DISTINCT FROM $2",
			&[id, &keep_session],
		)
		.await?;

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

	/// The user if the email and password are right. An unknown email takes
	/// as long as a wrong password.
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
			.unwrap_or_else(|| DUMMY_HASH.clone());
		let right = tokio::task::spawn_blocking(move || {
			bcrypt::verify(&password, &hash).unwrap_or(false)
		})
		.await
		.unwrap_or(false);

		match found.filter(|_| right) {
			Some(f) => self.by_id(&f.id).await,
			None => Ok(None),
		}
	}

	pub async fn new_session(&self, user_id: &UniqueId) -> Result<Session> {
		let conn = self.db.get().await?;

		let session = Session {
			token: Token::new(),
			timeout: Timeout::new(SESSION_TIMEOUT),
			user_id: *user_id,
		};
		conn.connection()
			.execute(
				"INSERT INTO user_sessions
				(token, timeout, user_id, created_on)
				VALUES ($1, $2, $3, $4)",
				&[
					&session.token,
					&session.timeout,
					&session.user_id,
					&DateTime::now(),
				],
			)
			.await?;

		Ok(session)
	}

	/// Returns the session if it exists and is not expired
	pub async fn session(&self, token: &Token) -> Result<Option<Session>> {
		let conn = self.db.get().await?;

		let session: Option<Session> = conn
			.connection()
			.query_opt(
				"SELECT token, timeout, user_id FROM user_sessions
				WHERE token = $1",
				&[token],
			)
			.await?;

		Ok(session.filter(|s| !s.timeout.has_elapsed()))
	}

	/// Replaces the session with a new one
	pub async fn renew_session(&self, session: Session) -> Result<Session> {
		self.delete_session(&session.token).await?;
		self.new_session(&session.user_id).await
	}

	pub async fn delete_session(&self, token: &Token) -> Result<()> {
		let conn = self.db.get().await?;

		conn.connection()
			.execute("DELETE FROM user_sessions WHERE token = $1", &[token])
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
}

#[derive(FromRow)]
struct IdRow {
	id: UniqueId,
}

#[derive(FromRow)]
struct LoginRow {
	id: UniqueId,
	password_hash: String,
}

fn session_token(parts: &Parts) -> Option<Token> {
	parts
		.headers
		.get("session-token")?
		.to_str()
		.ok()?
		.parse()
		.ok()
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

/// A valid session from the `session-token` header
pub struct AuthedSession(pub Session);

impl<S> FromRequestParts<S> for AuthedSession
where
	S: Send + Sync,
	Users: FromRef<S>,
{
	type Rejection = Error;

	async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
		let token = session_token(parts).ok_or(Error::InvalidSessionToken)?;

		Users::from_ref(state)
			.session(&token)
			.await?
			.map(Self)
			.ok_or(Error::InvalidSessionToken)
	}
}

/// The user of a valid session, only the webui
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
		let AuthedSession(session) =
			AuthedSession::from_request_parts(parts, state).await?;
		let user = Users::from_ref(state)
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
}
