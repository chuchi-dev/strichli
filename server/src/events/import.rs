//! Imports the history of Simple Analytics from its export api
//! (`/api/export/datapoints?version=6&format=ndjson&type=all`), one json
//! object per line with the fields `added_iso`, `path`, `is_unique`,
//! `is_robot`, `referrer_hostname`, `datapoint` and the event data as
//! `metadata.<field>_<type>`.

use chrono::{DateTime, Utc};
use serde_json::{Map, Value};

use super::{NewEvent, check_data, check_key, clean_path, external_referrer};

pub const SIMPLE_ANALYTICS: &str = "simpleanalytics";

/// The types Simple Analytics appends to metadata fields
const METADATA_TYPES: &[&str] = &["_text", "_int", "_bool", "_date"];

#[derive(Debug, Clone, PartialEq)]
pub struct ImportedEvent {
	pub ts: DateTime<Utc>,
	pub event: NewEvent,
}

/// One line of the export, none for robots and lines that don't fit (an
/// invalid path or key)
pub fn parse_line(line: &str, domain: &str) -> Option<ImportedEvent> {
	let obj: Map<String, Value> = serde_json::from_str(line).ok()?;
	let str_of = |name: &str| obj.get(name).and_then(Value::as_str);

	if obj.get("is_robot").and_then(Value::as_bool) == Some(true) {
		return None;
	}

	let ts = str_of("added_iso")?.parse().ok()?;
	let path = clean_path(str_of("path").unwrap_or("/")).ok()?;

	let datapoint = str_of("datapoint").unwrap_or("pageview");
	let event = if datapoint == "pageview" {
		NewEvent {
			path,
			referrer: external_referrer(str_of("referrer_hostname"), domain),
			is_unique: obj.get("is_unique").and_then(Value::as_bool)
				== Some(true),
			key: None,
			data: None,
			device: None,
		}
	} else {
		let key = check_key(datapoint).ok()?;
		let data: Map<String, Value> = obj
			.iter()
			.filter_map(|(name, v)| {
				let field = name.strip_prefix("metadata.")?;
				let field = METADATA_TYPES
					.iter()
					.find_map(|t| field.strip_suffix(t))
					.unwrap_or(field);
				Some((field.to_string(), v.clone()))
			})
			.filter(|(_, v)| !v.is_null())
			.collect();

		NewEvent {
			path,
			referrer: None,
			is_unique: false,
			key: Some(key),
			data: check_data(data).ok().flatten(),
			device: None,
		}
	};

	Some(ImportedEvent { ts, event })
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn pageviews() {
		let line = r#"{"added_iso":"2026-10-01T08:29:18.258Z","hostname":"soerenmeier.ch","path":"/blog/","is_unique":true,"is_robot":false,"referrer_hostname":"www.google.com","datapoint":"pageview"}"#;
		let ev = parse_line(line, "soerenmeier.ch").unwrap();

		assert_eq!(ev.ts.to_rfc3339(), "2026-10-01T08:29:18.258+00:00");
		assert_eq!(ev.event.path, "/blog");
		assert_eq!(ev.event.referrer.as_deref(), Some("google.com"));
		assert!(ev.event.is_unique);
		assert_eq!(ev.event.key, None);
	}

	#[test]
	fn own_referrer_is_dropped() {
		let line = r#"{"added_iso":"2026-10-01T08:29:18Z","path":"/","is_unique":false,"referrer_hostname":"soerenmeier.ch","datapoint":"pageview"}"#;
		let ev = parse_line(line, "soerenmeier.ch").unwrap();

		assert_eq!(ev.event.referrer, None);
		assert!(!ev.event.is_unique);
	}

	#[test]
	fn events_with_metadata() {
		let line = r#"{"added_iso":"2026-10-20T13:56:40.693Z","path":"/dns","datapoint":"lookup_domain","metadata.domain_text":"dunkel.cc","metadata.domain_int":null,"metadata.nope_text":null}"#;
		let ev = parse_line(line, "webtoolbelt.dev").unwrap();

		assert_eq!(ev.event.key.as_deref(), Some("lookup_domain"));
		assert_eq!(ev.event.data, Some(json!({ "domain": "dunkel.cc" })));
		assert!(!ev.event.is_unique);
	}

	#[test]
	fn skips_robots_and_broken_lines() {
		let robot = r#"{"added_iso":"2026-10-01T08:29:18Z","path":"/","is_robot":true,"datapoint":"pageview"}"#;
		assert_eq!(parse_line(robot, "a.ch"), None);
		assert_eq!(parse_line("not json", "a.ch"), None);
		assert_eq!(parse_line(r#"{"path":"/"}"#, "a.ch"), None);

		let bad_key = r#"{"added_iso":"2026-10-01T08:29:18Z","path":"/","datapoint":"has space"}"#;
		assert_eq!(parse_line(bad_key, "a.ch"), None);
	}
}
