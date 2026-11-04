//! In memory rate limits per ip
//!
//! The limits are only per second and minute, so there is no need to keep
//! them across restarts.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::convert::Infallible;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use tokio::time::interval;

use crate::error::Error;

/// How many requests an ip may send
#[derive(Debug, Clone, Copy)]
pub struct Allowed {
	pub second: u64,
	pub minute: u64,
}

#[derive(Debug, Clone)]
pub struct Limits {
	allowed: Allowed,
	inner: Arc<Mutex<Inner>>,
}

impl Limits {
	pub fn new(allowed: Allowed) -> Self {
		Self {
			allowed,
			inner: Arc::new(Mutex::new(Inner::new())),
		}
	}

	/// Counts a request from this ip, returns an error with the seconds to
	/// wait if the limit is reached.
	pub fn check(&self, ip: ClientIp) -> Result<(), Error> {
		match self.inner.lock().unwrap().request(ip.0, self.allowed) {
			Some(d) => Err(Error::LimitReached(d.as_secs().max(1))),
			None => Ok(()),
		}
	}

	pub fn cleanup_task(&self) {
		let this = self.clone();
		tokio::spawn(async move {
			let mut interval = interval(Duration::from_secs(60 * 5));
			interval.tick().await;

			loop {
				interval.tick().await;

				let removed = this.inner.lock().unwrap().cleanup();
				tracing::debug!("cleaned up {removed} limits");
			}
		});
	}
}

/// The ip of the client, genco's nginx appends it last to
/// `x-forwarded-for`. It is only used for the rate limits and never stored.
#[derive(Debug, Clone, Copy)]
pub struct ClientIp(pub IpAddr);

impl<S> FromRequestParts<S> for ClientIp
where
	S: Send + Sync,
{
	type Rejection = Infallible;

	async fn from_request_parts(
		parts: &mut Parts,
		_state: &S,
	) -> Result<Self, Self::Rejection> {
		let ip = parts
			.headers
			.get("x-forwarded-for")
			.and_then(|v| v.to_str().ok())
			.and_then(|v| v.rsplit(',').next())
			.and_then(|s| s.trim().parse().ok())
			.unwrap_or(IpAddr::from([127, 0, 0, 1]));

		Ok(Self(ip))
	}
}

const SECOND: Duration = Duration::from_secs(1);
const MINUTE: Duration = Duration::from_secs(60);

#[derive(Debug)]
struct Inner {
	second: LimitData,
	minute: LimitData,
}

impl Inner {
	fn new() -> Self {
		Self {
			second: LimitData::default(),
			minute: LimitData::default(),
		}
	}

	/// returns None if the request is allowed, else how long to wait
	fn request(&mut self, ip: IpAddr, allowed: Allowed) -> Option<Duration> {
		let sec = self.second.limit_reached(&ip).unwrap_or_default();
		let min = self.minute.limit_reached(&ip).unwrap_or_default();

		let time_to_wait = sec.max(min);
		if !time_to_wait.is_zero() {
			return Some(time_to_wait);
		}

		self.second.decrease(ip, allowed.second, SECOND);
		self.minute.decrease(ip, allowed.minute, MINUTE);

		None
	}

	fn cleanup(&mut self) -> usize {
		self.second.cleanup() + self.minute.cleanup()
	}
}

#[derive(Debug, Clone, Copy)]
struct Limit {
	remaining: u64,
	expire: Instant,
}

impl Limit {
	fn is_expired(&self) -> bool {
		Instant::now() >= self.expire
	}
}

#[derive(Debug, Default)]
struct LimitData {
	inner: HashMap<IpAddr, Limit>,
}

impl LimitData {
	fn limit_reached(&self, ip: &IpAddr) -> Option<Duration> {
		let limit = self.inner.get(ip)?;
		let now = Instant::now();
		if now >= limit.expire || limit.remaining > 0 {
			return None;
		}

		Some(limit.expire.duration_since(now))
	}

	fn decrease(&mut self, ip: IpAddr, init: u64, dur: Duration) {
		match self.inner.entry(ip) {
			Entry::Occupied(mut occ) => {
				let limit = occ.get_mut();

				if limit.is_expired() {
					limit.remaining = init;
					limit.expire = Instant::now() + dur;
				}

				limit.remaining = limit.remaining.saturating_sub(1);
			}
			Entry::Vacant(vac) => {
				vac.insert(Limit {
					remaining: init.saturating_sub(1),
					expire: Instant::now() + dur,
				});
			}
		}
	}

	fn cleanup(&mut self) -> usize {
		let prev_len = self.inner.len();
		self.inner.retain(|_, l| !l.is_expired());
		prev_len - self.inner.len()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const MS: Duration = Duration::from_millis(20);

	#[test]
	fn limit_reached() {
		let mut data = LimitData::default();
		let ip: IpAddr = "127.0.0.1".parse().unwrap();

		assert!(data.limit_reached(&ip).is_none());
		data.decrease(ip, 2, MS);

		assert!(data.limit_reached(&ip).is_none());
		data.decrease(ip, 2, MS);

		assert!(data.limit_reached(&ip).is_some());

		std::thread::sleep(MS);

		assert!(data.limit_reached(&ip).is_none());
		assert_eq!(data.cleanup(), 1);
	}

	#[test]
	fn request() {
		let mut inner = Inner::new();
		let ip = IpAddr::from([127, 0, 0, 1]);
		let allowed = Allowed {
			second: 1,
			minute: 10,
		};

		assert!(inner.request(ip, allowed).is_none());
		assert!(inner.request(ip, allowed).is_some());
	}
}
