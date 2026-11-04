// strichli: counts pageviews and keys without cookies, nothing is stored
// in the browser. Include it with
// <script async src="https://strichli.goodserver.ch/s.js"></script>
// and send keys with `strichli('key', { field: 'value' })`. Calls of simple
// analytics' `sa_event` keep working.
(function () {
	var w = window,
		d = document,
		l = location,
		nav = navigator,
		script = d.currentScript;
	if (!script || w.strichli) return;

	var endpoint = new URL('/e', script.src).href;
	var lastPath = null;

	function send(data) {
		// automated browsers don't count
		if (nav.webdriver) return;

		data.h = l.hostname;
		var body = JSON.stringify(data);
		if (nav.sendBeacon && nav.sendBeacon(endpoint, body)) return;

		fetch(endpoint, {
			method: 'POST',
			body: body,
			keepalive: true,
			mode: 'no-cors',
		}).catch(function () {});
	}

	function pageview(spa) {
		var path = l.pathname;
		if (path === lastPath) return;
		lastPath = path;

		var data = { p: path };
		if (spa) {
			data.s = true;
		} else {
			data.r = d.referrer;
			var perf = w.performance;
			var entry =
				perf && perf.getEntriesByType
					? perf.getEntriesByType('navigation')[0]
					: null;
			if (entry) data.n = entry.type;
		}

		send(data);
	}

	function key(name, data, callback) {
		if (typeof data === 'function') {
			callback = data;
			data = undefined;
		}

		send({ p: l.pathname, k: String(name), d: data });
		if (typeof callback === 'function') callback();
	}

	// single page apps
	var pushState = history.pushState;
	history.pushState = function () {
		var ret = pushState.apply(this, arguments);
		pageview(true);
		return ret;
	};
	w.addEventListener('popstate', function () {
		pageview(true);
	});

	w.strichli = key;
	if (!w.sa_event) w.sa_event = key;

	// a prerendered page counts once it is shown
	if (d.prerendering) {
		d.addEventListener(
			'prerenderingchange',
			function () {
				pageview(false);
			},
			{ once: true },
		);
	} else {
		pageview(false);
	}
})();
