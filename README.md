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
- `GET /api/sites/{id}/stats?from=YYYY-MM-DD&to=YYYY-MM-DD`
- `GET /api/sites/{id}/keys/{key}?from=…&to=…` the values of a key's data

Deleting a site needs a webui session.

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

In debug builds cors is enabled and `../webui/dist` is served. The dev
webui talks to `http://localhost:8080/api/`.

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
client ip for the rate limits from the last `x-forwarded-for` entry.

## License

MIT or Apache-2.0
