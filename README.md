# strichli

Cookieless analytics for our sites: visitors, pageviews, pages, referrers
and custom keys. No cookies, nothing stored in the browser, no ip or user
agent in the database, so no consent banner is needed.

- `server/` the axum server: the tracking script (`/s.js`), the collect
  endpoint (`/e`), the api under `/api` and the built webui
- `webui/` Svelte 5 + chuchi dashboard

## Tracking

```html
<script async src="https://strichli.goodserver.ch/s.js"></script>
```

Pageviews are sent on load and on every `history.pushState` (single page
apps). Keys:

```js
strichli('lookup_domain', { domain: 'example.com' });
```

`sa_event` (Simple Analytics) is defined too, so existing calls keep
working. strichli also answers Simple Analytics' paths `/latest.js` and
`/noscript.gif` (a pageview without javascript, from the referer, never a
visitor), so a custom Simple Analytics domain can point to strichli
without changing the sites. Keys are `[A-Za-z0-9_.-]`, up to 64 characters, data is a flat
object of strings, numbers and booleans (up to 1 KB).

A pageview counts as a visitor when it starts a visit: the referrer is not
the site itself, and it is no reload, back/forward or single page app
navigation (the method Simple Analytics uses). Bots are filtered by user
agent and `navigator.webdriver`.

## API

Admins create api tokens in the webui; a token acts as its admin on the
sites routes, send it as `authorization: Bearer <token>`.

- `GET /api/sites` sites with the last 30 days
- `POST /api/sites` `{ "domain", "name" }` adds a site, the response has
  the `snippet`
- `PUT /api/sites/{id}` `{ "domain", "name" }`
- `GET /api/sites/{id}/stats?range=7d` or `?from=YYYY-MM-DD&to=YYYY-MM-DD`:
  presets `today`, `7d`, `30d`, `90d`, `12m` count back from today in the
  configured timezone, at most 5 years; the answer has the `range` it shows
- `GET /api/sites/{id}/events?range=…&fields=domain,plan[&key=…]` the
  events grouped by key and the values of up to 5 data fields, with the
  fields there are

Deleting a site needs a webui session.

### Import from Simple Analytics

`POST /api/sites/{id}/import/simpleanalytics` (admins) takes the ndjson of
Simple Analytics' export api and replaces an earlier import of the site.
Only data points from before strichli's own first event are taken, so a
parallel run doesn't count twice. Robots are left out, event data is
requested per field as `metadata.<field>_text|_int|_bool`:

```sh
curl -s --get -H "Api-Key: $SA_KEY" \
	--data-urlencode hostname=soerenmeier.ch \
	--data fields=added_iso,path,is_unique,is_robot,referrer_hostname,datapoint,metadata.name_text \
	'https://simpleanalytics.com/api/export/datapoints?version=6&format=ndjson&type=all&start=2015-01-01&end=2030-01-01' \
	| curl -s -X POST https://strichli.goodserver.ch/api/sites/$SITE/import/simpleanalytics \
		-H "authorization: Bearer $TOKEN" -H 'content-type: text/plain' --data-binary @-
```

## Development

Requires rust, node and a postgres.

```sh
cd webui
npm ci
npm run dev

# in another shell, with a config.toml in server/
cd server
cargo run
# prints a link to set the password, `password-link --email` makes a new one
cargo run -- add-user --email you@example.com --name You --admin
```

In debug builds `../webui/dist` is served. The vite dev server passes
`/api` on to `http://127.0.0.1:8080`.

Formatting: `cargo fmt`, `npm run format`, checks: `cargo clippy`,
`cargo test`, `npm run check`.

## Config

```toml
# with protocol and trailing slash
public-addr = "https://strichli.goodserver.ch/"
# days and hours of the stats, the default
timezone = "Europe/Zurich"
# origins that may embed the webui in a frame, none by default
frame-ancestors = []

[database]
# a host or the directory of the unix socket
host = "/var/run/postgresql"
name = "strichli"
user = "strichli"
password = ""
```

The server listens on `HOST`:`PORT` (default `127.0.0.1:8080`) and takes the
client ip for the rate limits from the last `x-forwarded-for` entry (an
IPv6 client counts as its /64), so it must only be reachable through the
proxy.

## Privacy and security

- No cookies, nothing in the browser's storage. The database has no ip,
  user agent or visitor id; the ip only counts against in-memory limits.
- The proxy's access log still sees ip, user agent and referer of `/e`,
  `/s.js` and `/noscript.gif`; turn it off for them if that matters.
- Session tokens, api tokens and password links are 32 random bytes, only
  their sha256 is stored. Sessions end 30 days after the last use, at the
  latest a year after the login.
- Logins are limited per client and, after 10 failures in 15 minutes, per
  email. Events are limited per client and per site.
- Anyone can send events for a site (that is how the tracking works),
  the limits and the size checks keep it bounded.

## License

MIT or Apache-2.0
