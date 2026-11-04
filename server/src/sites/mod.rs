//! The sites strichli counts for. A site is a domain, events from the domain
//! and its www. subdomain count for it.

pub mod routes;

use std::collections::HashMap;

use pg::time::DateTime;
use pg::{Database, FromRow, UniqueId};
use serde::Serialize;

use crate::db::{IdRow, migrate};
use crate::error::{Error, Result};
use crate::users::User;

const MIGRATIONS: &[(&str, &str)] = &[(
	"create-sites",
	include_str!("./migrations/create-sites.sql"),
)];

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Site {
	pub id: UniqueId,
	pub domain: String,
	pub name: String,
	pub created_on: DateTime,
}

/// A domain or the host of a url: lowercase, without protocol, path, port
/// and www.
pub fn normalize_domain(s: &str) -> Result<String> {
	let s = s.trim().to_lowercase();
	let s = s
		.strip_prefix("https://")
		.or_else(|| s.strip_prefix("http://"))
		.unwrap_or(&s);
	let host = s.split(['/', '?', '#', ':']).next().unwrap_or("");
	let host = host.trim_end_matches('.');
	let host = host.strip_prefix("www.").unwrap_or(host);

	let valid = host.len() <= 253
		&& host.contains('.')
		&& host.split('.').all(|label| {
			!label.is_empty()
				&& label.len() <= 63
				&& !label.starts_with('-')
				&& !label.ends_with('-')
				&& label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
		});
	if !valid {
		return Err(Error::Invalid(format!("invalid domain {s:?}")));
	}

	Ok(host.into())
}

fn check_name(name: &str) -> Result<String> {
	let name = name.trim();
	if name.is_empty() {
		return Err(Error::Invalid("the name is empty".into()));
	}

	Ok(name.into())
}

const SITE_COLUMNS: &str = "s.id, s.domain, s.name, s.created_on";

#[derive(Clone)]
pub struct Sites {
	db: Database,
}

impl Sites {
	pub async fn new(db: &Database) -> Self {
		migrate(db, MIGRATIONS).await;

		Self { db: db.clone() }
	}

	/// Every site for admins, else the ones the user was given
	pub async fn list(&self, user: &User) -> Result<Vec<Site>> {
		let conn = self.db.get().await?;

		Ok(conn
			.connection()
			.query(
				&format!(
					"SELECT {SITE_COLUMNS} FROM sites s
					WHERE $1 OR EXISTS (
						SELECT 1 FROM site_users u
						WHERE u.site_id = s.id AND u.user_id = $2
					)
					ORDER BY s.name"
				),
				&[&user.admin, &user.id],
			)
			.await?)
	}

	/// The site if the user may see it
	pub async fn get(&self, user: &User, id: &UniqueId) -> Result<Site> {
		let conn = self.db.get().await?;

		let site: Option<Site> = conn
			.connection()
			.query_opt(
				&format!(
					"SELECT {SITE_COLUMNS} FROM sites s
					WHERE s.id = $1 AND ($2 OR EXISTS (
						SELECT 1 FROM site_users u
						WHERE u.site_id = s.id AND u.user_id = $3
					))"
				),
				&[id, &user.admin, &user.id],
			)
			.await?;

		site.ok_or(Error::NotFound)
	}

	/// The site of a domain from `normalize_domain`
	pub async fn by_domain(&self, domain: &str) -> Result<Option<UniqueId>> {
		let conn = self.db.get().await?;

		let row: Option<IdRow> = conn
			.connection()
			.query_opt("SELECT id FROM sites WHERE domain = $1", &[&domain])
			.await?;

		Ok(row.map(|r| r.id))
	}

	pub async fn create(&self, domain: &str, name: &str) -> Result<Site> {
		let site = Site {
			id: UniqueId::new(),
			domain: normalize_domain(domain)?,
			name: check_name(name)?,
			created_on: DateTime::now(),
		};
		let conn = self.db.get().await?;

		conn.connection()
			.execute(
				"INSERT INTO sites (id, domain, name, created_on)
				VALUES ($1, $2, $3, $4)",
				&[&site.id, &site.domain, &site.name, &site.created_on],
			)
			.await?;

		tracing::info!("created the site {} {}", site.domain, site.id);
		Ok(site)
	}

	pub async fn update(
		&self,
		id: &UniqueId,
		domain: &str,
		name: &str,
	) -> Result<Site> {
		let domain = normalize_domain(domain)?;
		let name = check_name(name)?;
		let conn = self.db.get().await?;

		conn.connection()
			.query_opt(
				&format!(
					"UPDATE sites s SET domain = $2, name = $3 WHERE s.id = $1
					RETURNING {SITE_COLUMNS}"
				),
				&[id, &domain, &name],
			)
			.await?
			.ok_or(Error::NotFound)
	}

	/// Its events go with it
	pub async fn delete(&self, id: &UniqueId) -> Result<()> {
		let conn = self.db.get().await?;

		let n = conn
			.connection()
			.execute("DELETE FROM sites WHERE id = $1", &[id])
			.await?;
		if n == 0 {
			return Err(Error::NotFound);
		}

		Ok(())
	}

	/// The sites each user was given, by user id
	pub async fn access(&self) -> Result<HashMap<UniqueId, Vec<UniqueId>>> {
		let conn = self.db.get().await?;

		let rows: Vec<AccessRow> = conn
			.connection()
			.query("SELECT site_id, user_id FROM site_users", &[])
			.await?;

		let mut access: HashMap<UniqueId, Vec<UniqueId>> = HashMap::new();
		for row in rows {
			access.entry(row.user_id).or_default().push(row.site_id);
		}

		Ok(access)
	}
}

#[derive(FromRow)]
struct AccessRow {
	site_id: UniqueId,
	user_id: UniqueId,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn normalizes_domains() {
		let ok = |s| normalize_domain(s).unwrap();
		assert_eq!(ok("soerenmeier.ch"), "soerenmeier.ch");
		assert_eq!(ok(" WWW.SoerenMeier.ch. "), "soerenmeier.ch");
		assert_eq!(
			ok("https://www.webtoolbelt.dev/dns?x=1"),
			"webtoolbelt.dev"
		);
		assert_eq!(ok("http://shop.example.com:8080/"), "shop.example.com");

		assert!(normalize_domain("localhost").is_err());
		assert!(normalize_domain("").is_err());
		assert!(normalize_domain("a..ch").is_err());
		assert!(normalize_domain("-a.ch").is_err());
		assert!(normalize_domain("a b.ch").is_err());
	}
}
