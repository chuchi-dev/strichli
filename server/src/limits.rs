//! In memory rate limits, per client ip or per site, and failed logins per
//! email.
//!
//! The limits are only per second, minute or a few minutes, so there is no
//! need to keep them across restarts.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::convert::Infallible;
use std::hash::Hash;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use tokio::time::interval;

use crate::error::Error;

/// How many requests a source may send
#[derive(Debug, Clone, Copy)]
pub struct Allowed {
	pub second: u64,
	pub minute: u64,
}

#[derive(Debug, Clone)]
pub struct Limits<K> {
	allowed: Allowed,
	inner: Arc<Mutex<Inner<K>>>,
}

impl<K> Limits<K>
where
	K: Hash + Eq + Copy + Send + 'static,
{
	pub fn new(allowed: Allowed) -> Self {
		Self {
			allowed,
			inner: Arc::new(Mutex::new(Inner::new())),
		}
	}

	/// Counts a request from this source, returns an error with the
	/// seconds to wait if the limit is reached.
	pub fn check(&self, source: K) -> Result<(), Error> {
		match self.inner.lock().unwrap().request(source, self.allowed) {
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

/// The client, genco's nginx appends its ip last to `x-forwarded-for` (the
/// app only listens on localhost). An IPv6 client counts as its /64, one
/// client usually has all of it. Only used for the limits, never stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientIp(pub IpAddr);

impl ClientIp {
	pub fn new(ip: IpAddr) -> Self {
		match ip {
			IpAddr::V4(_) => Self(ip),
			IpAddr::V6(v6) => {
				let bits = u128::from(v6) & !((1u128 << 64) - 1);
				Self(IpAddr::V6(bits.into()))
			}
		}
	}
}

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

		Ok(Self::new(ip))
	}
}

/// Failed logins per email: after `MAX_FAILURES` within `FAILURE_WINDOW`
/// the account waits, whatever address the attempts come from.
#[derive(Debug, Clone, Default)]
pub struct Failures {
	inner: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
}

const MAX_FAILURES: u32 = 10;
const FAILURE_WINDOW: Duration = Duration::from_secs(15 * 60);

impl Failures {
	/// Returns an error while the key has too many failures
	pub fn check(&self, key: &str) -> Result<(), Error> {
		let inner = self.inner.lock().unwrap();
		match inner.get(key) {
			Some((n, since))
				if *n >= MAX_FAILURES && since.elapsed() < FAILURE_WINDOW =>
			{
				let wait = FAILURE_WINDOW - since.elapsed();
				Err(Error::LimitReached(wait.as_secs().max(1)))
			}
			_ => Ok(()),
		}
	}

	pub fn failed(&self, key: &str) {
		let mut inner = self.inner.lock().unwrap();
		let entry = inner.entry(key.into()).or_insert((0, Instant::now()));
		if entry.1.elapsed() >= FAILURE_WINDOW {
			*entry = (0, Instant::now());
		}
		entry.0 += 1;
	}

	pub fn clear(&self, key: &str) {
		self.inner.lock().unwrap().remove(key);
	}

	pub fn cleanup_task(&self) {
		let this = self.clone();
		tokio::spawn(async move {
			let mut interval = interval(FAILURE_WINDOW);
			interval.tick().await;

			loop {
				interval.tick().await;
				this.inner
					.lock()
					.unwrap()
					.retain(|_, (_, since)| since.elapsed() < FAILURE_WINDOW);
			}
		});
	}
}

const SECOND: Duration = Duration::from_secs(1);
const MINUTE: Duration = Duration::from_secs(60);

#[derive(Debug)]
struct Inner<K> {
	second: LimitData<K>,
	minute: LimitData<K>,
}

impl<K: Hash + Eq + Copy> Inner<K> {
	fn new() -> Self {
		Self {
			second: LimitData::default(),
			minute: LimitData::default(),
		}
	}

	/// returns None if the request is allowed, else how long to wait
	fn request(&mut self, source: K, allowed: Allowed) -> Option<Duration> {
		let sec = self.second.limit_reached(&source).unwrap_or_default();
		let min = self.minute.limit_reached(&source).unwrap_or_default();

		let time_to_wait = sec.max(min);
		if !time_to_wait.is_zero() {
			return Some(time_to_wait);
		}

		self.second.decrease(source, allowed.second, SECOND);
		self.minute.decrease(source, allowed.minute, MINUTE);

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

#[derive(Debug)]
struct LimitData<K> {
	inner: HashMap<K, Limit>,
}

impl<K> Default for LimitData<K> {
	fn default() -> Self {
		Self {
			inner: HashMap::new(),
		}
	}
}

impl<K: Hash + Eq + Copy> LimitData<K> {
	fn limit_reached(&self, source: &K) -> Option<Duration> {
		let limit = self.inner.get(source)?;
		let now = Instant::now();
		if now >= limit.expire || limit.remaining > 0 {
			return None;
		}

		Some(limit.expire.duration_since(now))
	}

	fn decrease(&mut self, source: K, init: u64, dur: Duration) {
		match self.inner.entry(source) {
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

	#[test]
	fn ipv6_counts_as_its_64() {
		let a = ClientIp::new("2001:db8:1:2:aaaa::1".parse().unwrap());
		let b = ClientIp::new("2001:db8:1:2:ffff::9".parse().unwrap());
		let other = ClientIp::new("2001:db8:1:3::1".parse().unwrap());
		assert_eq!(a, b);
		assert_ne!(a, other);

		let v4 = ClientIp::new("192.0.2.7".parse().unwrap());
		assert_eq!(v4.0, "192.0.2.7".parse::<IpAddr>().unwrap());
	}

	#[test]
	fn failures_block_after_the_max() {
		let failures = Failures::default();
		for _ in 0..MAX_FAILURES {
			assert!(failures.check("a@b.ch").is_ok());
			failures.failed("a@b.ch");
		}
		assert!(failures.check("a@b.ch").is_err());
		assert!(failures.check("other@b.ch").is_ok());

		failures.clear("a@b.ch");
		assert!(failures.check("a@b.ch").is_ok());
	}
}
