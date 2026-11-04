//! Pageviews and keys (custom events) from the tracking script, and the
//! stats over them.
//!
//! Nothing that identifies a visitor is stored. A pageview counts as a
//! visitor when it starts a visit: it doesn't come from the site itself, it
//! is no reload, no back or forward and no navigation inside a single page
//! app. Days and hours are in the configured timezone.

pub mod import;
pub mod routes;

use std::collections::HashMap;

use pg::{Database, FromRow, UniqueId};
use postgres_types::ToSql;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::error::{Error, Result};
use crate::sites::normalize_domain;

const MIGRATIONS: &[(&str, &str)] = &[
	(
		"create-events",
		include_str!("./migrations/create-events.sql"),
	),
	("add-source", include_str!("./migrations/add-source.sql")),
];

const MAX_PATH: usize = 512;
const MAX_KEY: usize = 64;
const MAX_DATA_FIELDS: usize = 20;
const MAX_DATA_VALUE: usize = 256;
const MAX_DATA: usize = 1024;

/// Parts of user agents of bots, crawlers, link previews and tools
const BOTS: &[&str] = &[
	"bot",
	"crawl",
	"spider",
	"slurp",
	"headless",
	"lighthouse",
	"pingdom",
	"preview",
	"facebookexternalhit",
	"python",
	"curl/",
	"wget/",
	"go-http",
	"java/",
	"okhttp",
	"phantomjs",
	"selenium",
	"puppeteer",
	"playwright",
];

/// What the tracking script sends, short names keep it small
#[derive(Debug, Clone, Deserialize)]
pub struct Payload {
	/// hostname
	pub h: String,
	/// pathname
	pub p: String,
	/// `document.referrer`, only on the first pageview
	#[serde(default)]
	pub r: Option<String>,
	/// the navigation type of the first pageview (`navigate`, `reload`,
	/// `back_forward`)
	#[serde(default)]
	pub n: Option<String>,
	/// a navigation inside a single page app
	#[serde(default)]
	pub s: bool,
	/// the key of a custom event, none for a pageview
	#[serde(default)]
	pub k: Option<String>,
	/// the data of a custom event
	#[serde(default)]
	pub d: Option<Map<String, Value>>,
}

/// An event ready to be stored
#[derive(Debug, Clone, PartialEq)]
pub struct NewEvent {
	pub path: String,
	pub referrer: Option<String>,
	pub is_unique: bool,
	pub key: Option<String>,
	pub data: Option<Value>,
}

pub fn is_bot(user_agent: Option<&str>) -> bool {
	let Some(ua) = user_agent.filter(|ua| !ua.trim().is_empty()) else {
		return true;
	};
	let ua = ua.to_lowercase();

	BOTS.iter().any(|b| ua.contains(b))
}

fn clean_path(path: &str) -> Result<String> {
	if !path.starts_with('/') || path.len() > MAX_PATH {
		return Err(Error::Invalid("invalid path".into()));
	}

	let path = if path.len() > 1 {
		path.trim_end_matches('/')
	} else {
		path
	};
	Ok(if path.is_empty() { "/" } else { path }.into())
}

fn check_key(key: &str) -> Result<String> {
	let key = key.trim();
	let valid = !key.is_empty()
		&& key.len() <= MAX_KEY
		&& key
			.chars()
			.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
	if !valid {
		return Err(Error::Invalid(format!("invalid key {key:?}")));
	}

	Ok(key.into())
}

/// Only flat strings, numbers and booleans, nulls are dropped
fn check_data(data: Map<String, Value>) -> Result<Option<Value>> {
	let data: Map<String, Value> =
		data.into_iter().filter(|(_, v)| !v.is_null()).collect();
	if data.is_empty() {
		return Ok(None);
	}

	let invalid = |msg: &str| Err(Error::Invalid(msg.into()));
	if data.len() > MAX_DATA_FIELDS {
		return invalid("too many data fields");
	}
	for (field, value) in &data {
		if field.is_empty() || field.len() > MAX_KEY {
			return invalid("invalid data field");
		}
		match value {
			Value::String(s) if s.len() > MAX_DATA_VALUE => {
				return invalid("data value too long");
			}
			Value::String(_) | Value::Number(_) | Value::Bool(_) => {}
			_ => {
				return invalid(
					"data values must be strings, numbers or booleans",
				);
			}
		}
	}

	let data = Value::Object(data);
	if data.to_string().len() > MAX_DATA {
		return invalid("data too big");
	}

	Ok(Some(data))
}

/// A pageview without javascript, from the url of the page (the referer of
/// the noscript image). Returns the domain and the event. It can't tell
/// whether it starts a visit, so it is no visitor.
pub fn noscript_pageview(page_url: &str) -> Result<(String, NewEvent)> {
	let rest = page_url
		.split_once("://")
		.map(|(_, rest)| rest)
		.ok_or_else(|| Error::Invalid("invalid page url".into()))?;
	let (host, path) = match rest.find(['/', '?', '#']) {
		Some(i) => rest.split_at(i),
		None => (rest, "/"),
	};
	let path = path
		.split(['?', '#'])
		.next()
		.filter(|p| p.starts_with('/'))
		.unwrap_or("/");

	let event = NewEvent {
		path: clean_path(path)?,
		referrer: None,
		is_unique: false,
		key: None,
		data: None,
	};
	Ok((normalize_domain(host)?, event))
}

/// The host of the referrer, none if there is none or it's the site itself
fn external_referrer(referrer: Option<&str>, domain: &str) -> Option<String> {
	let host = normalize_domain(referrer?).ok()?;

	(host != domain).then_some(host)
}

impl Payload {
	/// The event for the site with this domain
	pub fn into_event(self, domain: &str) -> Result<NewEvent> {
		let path = clean_path(&self.p)?;

		if let Some(key) = self.k {
			return Ok(NewEvent {
				path,
				referrer: None,
				is_unique: false,
				key: Some(check_key(&key)?),
				data: self.d.map(check_data).transpose()?.flatten(),
			});
		}

		let referrer = external_referrer(self.r.as_deref(), domain);
		let from_site = self
			.r
			.as_deref()
			.and_then(|r| normalize_domain(r).ok())
			.is_some_and(|host| host == domain);
		let revisit =
			matches!(self.n.as_deref(), Some("reload" | "back_forward"));

		Ok(NewEvent {
			path,
			referrer: referrer.filter(|_| !self.s),
			is_unique: !self.s && !from_site && !revisit,
			key: None,
			data: None,
		})
	}
}

/// A range of days, `YYYY-MM-DD`, both included
#[derive(Debug, Clone, Deserialize)]
pub struct Range {
	pub from: String,
	pub to: String,
}

fn check_date(date: &str) -> bool {
	let parts: Vec<&str> = date.split('-').collect();
	let [y, m, d] = parts.as_slice() else {
		return false;
	};
	let num = |s: &str, len: usize| {
		(s.len() == len && s.chars().all(|c| c.is_ascii_digit()))
			.then(|| s.parse::<u32>().ok())
			.flatten()
	};

	matches!(
		(num(y, 4), num(m, 2), num(d, 2)),
		(Some(_), Some(1..=12), Some(1..=31))
	)
}

impl Range {
	pub fn check(&self) -> Result<()> {
		if !check_date(&self.from) || !check_date(&self.to) {
			return Err(Error::Invalid("dates must be YYYY-MM-DD".into()));
		}
		// iso dates compare like strings
		if self.from > self.to {
			return Err(Error::Invalid("from is after to".into()));
		}

		Ok(())
	}

	/// a single day is shown in hours
	fn hourly(&self) -> bool {
		self.from == self.to
	}
}

/// The events in the range: `$2` from, `$3` to, `$4` the timezone
const IN_RANGE: &str = "\
	e.ts >= ($2::text::date::timestamp AT TIME ZONE $4::text) \
	AND e.ts < (($3::text::date + 1)::timestamp AT TIME ZONE $4::text)";

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Bucket {
	/// `YYYY-MM-DD` or `YYYY-MM-DD HH:00`
	pub date: String,
	pub visitors: i64,
	pub pageviews: i64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PageCount {
	pub path: String,
	pub pageviews: i64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ReferrerCount {
	/// none for direct visits
	pub referrer: Option<String>,
	pub visitors: i64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct KeyCount {
	pub key: String,
	pub count: i64,
}

/// A key with the values of the chosen data fields and how often it was
/// sent with them
#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct EventRow {
	pub key: String,
	/// field to value, null if the event had no such field
	pub values: Value,
	pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventTable {
	/// the data fields of the keys in the range, to choose columns from
	pub fields: Vec<String>,
	pub rows: Vec<EventRow>,
}

/// Which events and how to group them
#[derive(Debug, Clone, Deserialize)]
pub struct EventQuery {
	#[serde(flatten)]
	pub range: Range,
	/// data fields to group by, comma separated
	#[serde(default)]
	pub fields: String,
	/// only this key
	#[serde(default)]
	pub key: Option<String>,
}

const MAX_COLUMNS: usize = 5;
const MAX_ROWS: usize = 1000;

impl EventQuery {
	fn columns(&self) -> Result<Vec<String>> {
		let columns: Vec<String> = self
			.fields
			.split(',')
			.map(str::trim)
			.filter(|f| !f.is_empty())
			.map(String::from)
			.collect();

		if columns.len() > MAX_COLUMNS {
			return Err(Error::Invalid(format!(
				"at most {MAX_COLUMNS} fields"
			)));
		}
		if columns.iter().any(|c| c.len() > MAX_KEY) {
			return Err(Error::Invalid("invalid field".into()));
		}

		Ok(columns)
	}
}

/// How an import went
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
	/// events of the earlier import of the same source, deleted
	pub replaced: u64,
	pub imported: u64,
	/// events from when strichli already counted itself, left out
	pub overlapping: u64,
}

const IMPORT_BATCH: usize = 1000;

#[derive(FromRow)]
struct FieldRow {
	field: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
	pub visitors: i64,
	pub pageviews: i64,
	pub hourly: bool,
	pub buckets: Vec<Bucket>,
	pub pages: Vec<PageCount>,
	pub referrers: Vec<ReferrerCount>,
	pub keys: Vec<KeyCount>,
}

#[derive(FromRow)]
struct SiteBucket {
	site_id: String,
	date: String,
	visitors: i64,
	pageviews: i64,
}

#[derive(Clone)]
pub struct Events {
	db: Database,
	timezone: String,
}

impl Events {
	pub async fn new(db: &Database, timezone: &str) -> Self {
		let migrations = db.migrations();
		let mut conn = db.get().await.unwrap();

		for (name, sql) in MIGRATIONS {
			migrations
				.add(&mut conn, name, sql)
				.await
				.expect("failed to run migration");
		}

		conn.connection()
			.execute("SELECT now() AT TIME ZONE $1::text", &[&timezone])
			.await
			.expect("unknown timezone in config.toml");

		Self {
			db: db.clone(),
			timezone: timezone.into(),
		}
	}

	pub async fn insert(
		&self,
		site_id: &UniqueId,
		ev: &NewEvent,
	) -> Result<()> {
		let conn = self.db.get().await?;

		conn.connection()
			.execute(
				"INSERT INTO events
				(site_id, path, referrer, is_unique, key, data)
				VALUES ($1, $2, $3, $4, $5, $6)",
				&[
					site_id,
					&ev.path,
					&ev.referrer,
					&ev.is_unique,
					&ev.key,
					&ev.data,
				],
			)
			.await?;

		Ok(())
	}

	/// The last 30 days of each site, by site id, today last
	pub async fn last_30_days(
		&self,
		site_ids: &[UniqueId],
	) -> Result<HashMap<String, Vec<Bucket>>> {
		let ids: Vec<String> =
			site_ids.iter().map(|id| id.to_string()).collect();
		let conn = self.db.get().await?;

		let rows: Vec<SiteBucket> = conn
			.connection()
			.query(
				"WITH today AS (
					SELECT (now() AT TIME ZONE $1::text)::date AS day
				)
				SELECT s.id AS site_id,
					to_char(d.day, 'YYYY-MM-DD') AS date,
					COALESCE(c.visitors, 0) AS visitors,
					COALESCE(c.pageviews, 0) AS pageviews
				FROM sites s
				CROSS JOIN today
				CROSS JOIN generate_series(
					(today.day - 29)::timestamp, today.day::timestamp,
					interval '1 day'
				) AS d(day)
				LEFT JOIN (
					SELECT e.site_id,
						date_trunc('day', e.ts AT TIME ZONE $1::text) AS day,
						count(*) FILTER (WHERE e.is_unique) AS visitors,
						count(*) AS pageviews
					FROM events e, today
					WHERE e.key IS NULL AND e.site_id = ANY($2::text[])
						AND e.ts >= ((today.day - 29)::timestamp
							AT TIME ZONE $1::text)
					GROUP BY 1, 2
				) c ON c.site_id = s.id AND c.day = d.day
				WHERE s.id = ANY($2::text[])
				ORDER BY s.id, d.day",
				&[&self.timezone, &ids],
			)
			.await?;

		let mut days: HashMap<String, Vec<Bucket>> = HashMap::new();
		for row in rows {
			days.entry(row.site_id).or_default().push(Bucket {
				date: row.date,
				visitors: row.visitors,
				pageviews: row.pageviews,
			});
		}

		Ok(days)
	}

	pub async fn stats(
		&self,
		site_id: &UniqueId,
		range: &Range,
	) -> Result<Stats> {
		range.check()?;
		let conn = self.db.get().await?;
		let conn = conn.connection();
		let params: [&(dyn ToSql + Sync); 4] =
			[site_id, &range.from, &range.to, &self.timezone];

		let (unit, label) = if range.hourly() {
			("hour", "YYYY-MM-DD HH24:00")
		} else {
			("day", "YYYY-MM-DD")
		};
		let buckets: Vec<Bucket> = conn
			.query(
				&format!(
					"SELECT to_char(d.at, '{label}') AS date,
						COALESCE(c.visitors, 0) AS visitors,
						COALESCE(c.pageviews, 0) AS pageviews
					FROM generate_series(
						$2::text::date::timestamp,
						-- not into the future
						LEAST(
							$3::text::date::timestamp + interval '23 hours',
							date_trunc('{unit}', now() AT TIME ZONE $4::text)
						),
						interval '1 {unit}'
					) AS d(at)
					LEFT JOIN (
						SELECT date_trunc('{unit}', e.ts AT TIME ZONE $4::text)
								AS at,
							count(*) FILTER (WHERE e.is_unique) AS visitors,
							count(*) AS pageviews
						FROM events e
						WHERE e.site_id = $1 AND e.key IS NULL AND {IN_RANGE}
						GROUP BY 1
					) c ON c.at = d.at
					ORDER BY d.at"
				),
				&params,
			)
			.await?;

		let pages: Vec<PageCount> = conn
			.query(
				&format!(
					"SELECT e.path, count(*) AS pageviews FROM events e
					WHERE e.site_id = $1 AND e.key IS NULL AND {IN_RANGE}
					GROUP BY e.path ORDER BY pageviews DESC, e.path
					LIMIT 100"
				),
				&params,
			)
			.await?;

		let referrers: Vec<ReferrerCount> = conn
			.query(
				&format!(
					"SELECT e.referrer, count(*) AS visitors FROM events e
					WHERE e.site_id = $1 AND e.key IS NULL AND e.is_unique
						AND {IN_RANGE}
					GROUP BY e.referrer ORDER BY visitors DESC, e.referrer
					LIMIT 100"
				),
				&params,
			)
			.await?;

		let keys: Vec<KeyCount> = conn
			.query(
				&format!(
					"SELECT e.key, count(*) AS count FROM events e
					WHERE e.site_id = $1 AND e.key IS NOT NULL AND {IN_RANGE}
					GROUP BY e.key ORDER BY count DESC, e.key"
				),
				&params,
			)
			.await?;

		Ok(Stats {
			visitors: buckets.iter().map(|b| b.visitors).sum(),
			pageviews: buckets.iter().map(|b| b.pageviews).sum(),
			hourly: range.hourly(),
			buckets,
			pages,
			referrers,
			keys,
		})
	}

	/// Replaces the events of an earlier import from this source. Only
	/// events from before strichli's own first one are taken, so nothing
	/// counts twice.
	pub async fn import(
		&self,
		site_id: &UniqueId,
		source: &str,
		events: &[import::ImportedEvent],
	) -> Result<ImportResult> {
		let mut conn = self.db.get().await?;
		let trans = conn.transaction().await?;
		let conn = trans.connection();

		let replaced = conn
			.execute(
				"DELETE FROM events WHERE site_id = $1 AND source = $2",
				&[site_id, &source],
			)
			.await?;

		let mut imported = 0;
		for batch in events.chunks(IMPORT_BATCH) {
			let ts: Vec<String> =
				batch.iter().map(|e| e.ts.to_rfc3339()).collect();
			let path: Vec<&str> =
				batch.iter().map(|e| e.event.path.as_str()).collect();
			let referrer: Vec<Option<&str>> =
				batch.iter().map(|e| e.event.referrer.as_deref()).collect();
			let unique: Vec<bool> =
				batch.iter().map(|e| e.event.is_unique).collect();
			let key: Vec<Option<&str>> =
				batch.iter().map(|e| e.event.key.as_deref()).collect();
			let data: Vec<Option<String>> = batch
				.iter()
				.map(|e| e.event.data.as_ref().map(Value::to_string))
				.collect();

			imported += conn
				.execute(
					"INSERT INTO events
					(site_id, ts, path, referrer, is_unique, key, data, source)
					SELECT $1, t.ts::timestamptz, t.path, t.referrer,
						t.is_unique, t.key, t.data::jsonb, $2
					FROM unnest(
						$3::text[], $4::text[], $5::text[], $6::bool[],
						$7::text[], $8::text[]
					) AS t(ts, path, referrer, is_unique, key, data)
					WHERE t.ts::timestamptz < COALESCE(
						(SELECT min(e.ts) FROM events e
						WHERE e.site_id = $1 AND e.source IS NULL),
						'infinity'
					)",
					&[
						site_id, &source, &ts, &path, &referrer, &unique, &key,
						&data,
					],
				)
				.await?;
		}

		trans.commit().await?;

		Ok(ImportResult {
			replaced,
			imported,
			overlapping: events.len() as u64 - imported,
		})
	}

	/// The keys grouped by the values of the chosen data fields, the most
	/// frequent first
	pub async fn events(
		&self,
		site_id: &UniqueId,
		query: &EventQuery,
	) -> Result<EventTable> {
		query.range.check()?;
		let columns = query.columns()?;
		let conn = self.db.get().await?;
		let conn = conn.connection();

		// $1 to $4 for the range, then the key, then the fields
		let mut params: Vec<&(dyn ToSql + Sync)> =
			vec![site_id, &query.range.from, &query.range.to, &self.timezone];
		params.push(&query.key);
		let values = if columns.is_empty() {
			"'{}'::jsonb".to_string()
		} else {
			let pairs: Vec<String> = (0..columns.len())
				.map(|i| format!("${n}::text, e.data -> ${n}::text", n = i + 6))
				.collect();
			format!("jsonb_build_object({})", pairs.join(", "))
		};
		for c in &columns {
			params.push(c);
		}

		let rows: Vec<EventRow> = conn
			.query(
				&format!(
					"SELECT e.key, {values} AS values, count(*) AS count
					FROM events e
					WHERE e.site_id = $1 AND e.key IS NOT NULL
						AND ($5::text IS NULL OR e.key = $5::text)
						AND {IN_RANGE}
					GROUP BY 1, 2 ORDER BY count DESC, 1
					LIMIT {MAX_ROWS}"
				),
				&params,
			)
			.await?;

		let fields: Vec<FieldRow> = conn
			.query(
				&format!(
					"SELECT DISTINCT jsonb_object_keys(e.data) AS field
					FROM events e
					WHERE e.site_id = $1 AND e.data IS NOT NULL
						AND ($5::text IS NULL OR e.key = $5::text)
						AND {IN_RANGE}
					ORDER BY 1"
				),
				&params[..5],
			)
			.await?;

		Ok(EventTable {
			fields: fields.into_iter().map(|f| f.field).collect(),
			rows,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	fn payload(v: Value) -> Payload {
		serde_json::from_value(v).unwrap()
	}

	#[test]
	fn detects_bots() {
		assert!(is_bot(None));
		assert!(is_bot(Some(" ")));
		assert!(is_bot(Some("Mozilla/5.0 (compatible; Googlebot/2.1)")));
		assert!(is_bot(Some("Mozilla/5.0 HeadlessChrome/120.0")));
		assert!(is_bot(Some("curl/8.5.0")));
		assert!(!is_bot(Some(
			"Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0) AppleWebKit/605.1.15 \
			(KHTML, like Gecko) Version/17.0 Safari/605.1.15"
		)));
	}

	#[test]
	fn first_pageview_from_outside_is_a_visitor() {
		let ev = payload(json!({
			"h": "soerenmeier.ch", "p": "/blog/",
			"r": "https://www.google.com/", "n": "navigate"
		}))
		.into_event("soerenmeier.ch")
		.unwrap();

		assert_eq!(ev.path, "/blog");
		assert_eq!(ev.referrer.as_deref(), Some("google.com"));
		assert!(ev.is_unique);
	}

	#[test]
	fn direct_visit_is_a_visitor() {
		let ev = payload(json!({ "h": "a.ch", "p": "/", "r": "" }))
			.into_event("a.ch")
			.unwrap();

		assert_eq!(ev.path, "/");
		assert_eq!(ev.referrer, None);
		assert!(ev.is_unique);
	}

	#[test]
	fn same_site_reload_and_spa_are_no_visitors() {
		let from_site = payload(json!({
			"h": "a.ch", "p": "/x", "r": "https://www.a.ch/"
		}));
		let reload = payload(json!({ "h": "a.ch", "p": "/x", "n": "reload" }));
		let back =
			payload(json!({ "h": "a.ch", "p": "/x", "n": "back_forward" }));
		let spa = payload(json!({ "h": "a.ch", "p": "/x", "s": true }));

		for p in [from_site, reload, back, spa] {
			let ev = p.into_event("a.ch").unwrap();
			assert!(!ev.is_unique);
			assert_eq!(ev.referrer, None);
		}
	}

	#[test]
	fn keys_with_data() {
		let ev = payload(json!({
			"h": "webtoolbelt.dev", "p": "/dns", "k": "lookup_domain",
			"d": { "domain": "example.com", "n": 2, "empty": null }
		}))
		.into_event("webtoolbelt.dev")
		.unwrap();

		assert_eq!(ev.key.as_deref(), Some("lookup_domain"));
		assert_eq!(ev.data, Some(json!({ "domain": "example.com", "n": 2 })));
		assert!(!ev.is_unique);

		let no_data = payload(json!({ "h": "a.ch", "p": "/", "k": "login" }))
			.into_event("a.ch")
			.unwrap();
		assert_eq!(no_data.data, None);
	}

	#[test]
	fn rejects_bad_input() {
		let bad = [
			json!({ "h": "a.ch", "p": "no-slash" }),
			json!({ "h": "a.ch", "p": format!("/{}", "x".repeat(600)) }),
			json!({ "h": "a.ch", "p": "/", "k": "has space" }),
			json!({ "h": "a.ch", "p": "/", "k": "x", "d": { "a": { "b": 1 } } }),
			json!({ "h": "a.ch", "p": "/", "k": "x", "d": { "a": "x".repeat(300) } }),
		];

		for v in bad {
			assert!(payload(v).into_event("a.ch").is_err());
		}
	}

	#[test]
	fn noscript_pageviews() {
		let (domain, ev) =
			noscript_pageview("https://www.soerenmeier.ch/blog/?x=1#top")
				.unwrap();
		assert_eq!(domain, "soerenmeier.ch");
		assert_eq!(ev.path, "/blog");
		assert!(!ev.is_unique);

		let (_, ev) = noscript_pageview("https://a.ch?x=1").unwrap();
		assert_eq!(ev.path, "/");

		assert!(noscript_pageview("").is_err());
		assert!(noscript_pageview("https://localhost/").is_err());
	}

	#[test]
	fn checks_ranges() {
		let range = |from: &str, to: &str| Range {
			from: from.into(),
			to: to.into(),
		};

		assert!(range("2026-11-01", "2026-11-04").check().is_ok());
		assert!(range("2026-11-04", "2026-11-04").hourly());
		assert!(range("2026-11-05", "2026-11-04").check().is_err());
		assert!(range("2026-13-01", "2026-13-02").check().is_err());
		assert!(range("26-11-01", "2026-11-02").check().is_err());
		assert!(range("2026-11-01'", "2026-11-02").check().is_err());
	}
}
