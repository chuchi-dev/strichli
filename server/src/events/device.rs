//! The kind of device, the browser and the os from a user agent, coarse
//! enough that they say nothing about a single visitor. The user agent
//! itself is never stored.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Device {
	/// `mobile`, `tablet` or `desktop`
	pub kind: &'static str,
	pub browser: &'static str,
	pub os: &'static str,
}

/// The first match wins, so the ones that also name others come first:
/// Edge and Opera say Chrome, Chrome says Safari.
const BROWSERS: &[(&[&str], &str)] = &[
	(&["edg/", "edga/", "edgios/"], "Edge"),
	(&["opr/", "opera"], "Opera"),
	(&["samsungbrowser"], "Samsung Internet"),
	(&["firefox/", "fxios/"], "Firefox"),
	(&["chrome/", "crios/", "chromium/"], "Chrome"),
	(&["safari/"], "Safari"),
];

/// iOS says "like Mac OS X" and Android says Linux, so they come first
const OSES: &[(&[&str], &str)] = &[
	(&["iphone", "ipad", "ipod"], "iOS"),
	(&["android"], "Android"),
	(&["windows"], "Windows"),
	(&["cros"], "ChromeOS"),
	(&["macintosh", "mac os x"], "macOS"),
	(&["linux"], "Linux"),
];

fn first(ua: &str, list: &[(&[&str], &'static str)]) -> &'static str {
	list.iter()
		.find(|(parts, _)| parts.iter().any(|p| ua.contains(p)))
		.map_or("Other", |(_, name)| name)
}

impl Device {
	/// `touch`: the page saw a touch screen, which tells an iPad (that
	/// says Macintosh) from a Mac
	pub fn from_agent(user_agent: &str, touch: bool) -> Self {
		let ua = user_agent.to_lowercase();
		let mut os = first(&ua, OSES);
		let browser = first(&ua, BROWSERS);

		let kind = if ua.contains("ipad")
			|| ua.contains("tablet")
			|| (os == "Android" && !ua.contains("mobile"))
		{
			"tablet"
		} else if os == "macOS" && touch {
			os = "iOS";
			"tablet"
		} else if ua.contains("mobi")
			|| ua.contains("iphone")
			|| ua.contains("ipod")
		{
			"mobile"
		} else {
			"desktop"
		};

		Self { kind, browser, os }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn device(
		ua: &str,
		touch: bool,
	) -> (&'static str, &'static str, &'static str) {
		let d = Device::from_agent(ua, touch);
		(d.kind, d.browser, d.os)
	}

	#[test]
	fn common_agents() {
		let cases = [
			(
				"Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) \
				AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 \
				Mobile/15E148 Safari/604.1",
				("mobile", "Safari", "iOS"),
			),
			(
				"Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 \
				(KHTML, like Gecko) Chrome/130.0.0.0 Mobile Safari/537.36",
				("mobile", "Chrome", "Android"),
			),
			(
				"Mozilla/5.0 (Linux; Android 14; SM-X710) AppleWebKit/537.36 \
				(KHTML, like Gecko) SamsungBrowser/26.0 Chrome/122.0.0.0 \
				Safari/537.36",
				("tablet", "Samsung Internet", "Android"),
			),
			(
				"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
				(KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 \
				Edg/130.0.0.0",
				("desktop", "Edge", "Windows"),
			),
			(
				"Mozilla/5.0 (Macintosh; Intel Mac OS X 14.7; rv:132.0) \
				Gecko/20100101 Firefox/132.0",
				("desktop", "Firefox", "macOS"),
			),
			(
				"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, \
				like Gecko) Chrome/130.0.0.0 Safari/537.36 OPR/115.0.0.0",
				("desktop", "Opera", "Linux"),
			),
			(
				"Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) \
				AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/130.0 \
				Mobile/15E148 Safari/604.1",
				("mobile", "Chrome", "iOS"),
			),
			("something else", ("desktop", "Other", "Other")),
		];

		for (ua, expected) in cases {
			assert_eq!(device(ua, false), expected, "{ua}");
		}
	}

	#[test]
	fn ipad_says_mac() {
		let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
			AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 \
			Safari/605.1.15";

		assert_eq!(device(ua, false), ("desktop", "Safari", "macOS"));
		assert_eq!(device(ua, true), ("tablet", "Safari", "iOS"));
	}
}
