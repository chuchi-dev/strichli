mod error;
mod events;
mod headers;
mod limits;
mod sites;
mod users;
mod webui;

use std::fs;
use std::sync::Arc;

use axum::Router;
use axum::extract::FromRef;
use axum::http::{Method, header};
use axum::middleware::map_response_with_state;
use clap::{Parser, Subcommand};
use pg::Database;
use serde::Deserialize;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{self, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::error::Error;
use crate::events::Events;
use crate::headers::SecurityHeaders;
use crate::limits::{Allowed, Limits};
use crate::sites::Sites;
use crate::users::routes::password_url;
use crate::users::{NO_PASSWORD, Users};
use crate::webui::Webui;

#[derive(Debug, Parser)]
#[command(version, about = "Cookieless analytics")]
struct Args {
	#[arg(long, default_value = "strichli=info,warn")]
	tracing: String,
	#[arg(long, default_value = "config.toml")]
	config: String,
	#[arg(long, env = "HOST", default_value = "127.0.0.1")]
	host: String,
	#[arg(long, env = "PORT", default_value = "8080")]
	port: u16,
	#[arg(long, default_value_t = default_dist_dir())]
	dist_dir: String,
	#[arg(long)]
	enable_cors: bool,
	/// Without a command the server runs
	#[command(subcommand)]
	cmd: Option<Cmd>,
}

#[derive(Debug, Subcommand)]
enum Cmd {
	/// Adds a user and prints a link to set the password
	AddUser {
		#[arg(long)]
		email: String,
		#[arg(long)]
		name: String,
		/// sees and manages every site
		#[arg(long)]
		admin: bool,
	},
	/// Prints a new link to set the password of a user
	PasswordLink {
		#[arg(long)]
		email: String,
	},
}

fn default_dist_dir() -> String {
	if cfg!(debug_assertions) {
		"../webui/dist".into()
	} else {
		"webui/dist".into()
	}
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
	/// The public address, with protocol and trailing slash, for example
	/// `https://strichli.goodserver.ch/`
	public_addr: String,
	/// Days and hours of the stats are in this timezone
	#[serde(default = "default_timezone")]
	timezone: String,
	/// Origins that may embed the webui in a frame, none by default
	#[serde(default)]
	frame_ancestors: Vec<String>,
	database: DbConf,
}

fn default_timezone() -> String {
	"Europe/Zurich".into()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct DbConf {
	/// a host or the directory of the unix socket
	host: String,
	name: String,
	user: String,
	#[serde(default)]
	password: String,
}

#[derive(Clone)]
pub struct AppState {
	cfg: Arc<Config>,
	users: Users,
	sites: Sites,
	events: Events,
	/// events per ip
	collect_limits: Limits,
	/// login attempts per ip
	login_limits: Limits,
	webui: Webui,
}

impl FromRef<AppState> for Users {
	fn from_ref(state: &AppState) -> Self {
		state.users.clone()
	}
}

impl FromRef<AppState> for Webui {
	fn from_ref(state: &AppState) -> Self {
		state.webui.clone()
	}
}

fn create_app(state: AppState, enable_cors: bool) -> Router {
	let api = Router::new()
		.merge(users::routes::routes())
		.nest("/sites", sites::routes::routes());

	let mut app = Router::new()
		.nest("/api", api)
		.merge(events::routes::routes())
		.merge(webui::routes())
		.layer(map_response_with_state(
			SecurityHeaders::new(&state.cfg.frame_ancestors),
			headers::security_headers,
		))
		.layer(CompressionLayer::new())
		.layer(TraceLayer::new_for_http());

	if enable_cors {
		app = app.layer(
			CorsLayer::new()
				.allow_origin(cors::Any)
				.allow_methods([
					Method::GET,
					Method::POST,
					Method::PUT,
					Method::DELETE,
					Method::OPTIONS,
				])
				.allow_headers([
					header::CONTENT_TYPE,
					header::AUTHORIZATION,
					"session-token".parse().unwrap(),
				]),
		);
	}

	app.with_state(state)
}

#[tokio::main]
async fn main() {
	let args = Args::parse();

	tracing_subscriber::fmt()
		.with_env_filter(args.tracing)
		.init();

	let cfg_string =
		fs::read_to_string(&args.config).expect("failed to read config.toml");
	let cfg: Config =
		toml::from_str(&cfg_string).expect("failed to parse config.toml");
	let cfg = Arc::new(cfg);

	let db_cfg = &cfg.database;
	let database = Database::with_host(
		&db_cfg.host,
		&db_cfg.name,
		&db_cfg.user,
		&db_cfg.password,
	)
	.await
	.expect("could not connect to the database");

	// the order matters, the tables reference each other
	let users = Users::new(&database).await;
	let sites = Sites::new(&database).await;
	let events = Events::new(&database, &cfg.timezone).await;

	if let Some(cmd) = args.cmd {
		if let Err(e) = run_cmd(cmd, &users, &cfg).await {
			eprintln!("{e}");
			std::process::exit(1);
		}
		return;
	}

	let collect_limits = Limits::new(Allowed {
		second: 10,
		minute: 120,
	});
	let login_limits = Limits::new(Allowed {
		second: 1,
		minute: 10,
	});

	let state = AppState {
		cfg,
		users,
		sites,
		events,
		collect_limits: collect_limits.clone(),
		login_limits: login_limits.clone(),
		webui: Webui::new(&args.dist_dir),
	};

	collect_limits.cleanup_task();
	login_limits.cleanup_task();

	let app = create_app(state, args.enable_cors || cfg!(debug_assertions));

	let listen = format!("{}:{}", args.host, args.port);
	let listener = TcpListener::bind(&listen)
		.await
		.expect("failed to bind listen address");

	tracing::info!("listening on {listen}");

	axum::serve(listener, app)
		.with_graceful_shutdown(shutdown_signal())
		.await
		.unwrap();
}

async fn run_cmd(cmd: Cmd, users: &Users, cfg: &Config) -> error::Result<()> {
	let user = match cmd {
		Cmd::AddUser { email, name, admin } => {
			let user = users.create(&email, &name, NO_PASSWORD, admin).await?;
			println!("added {} ({})", user.email, user.id);
			user
		}
		Cmd::PasswordLink { email } => users
			.by_email(&email)
			.await?
			.ok_or_else(|| Error::Invalid(format!("no user {email}")))?,
	};

	let token = users.create_password_link(&user.id).await?;
	println!(
		"set the password within 7 days: {}",
		password_url(&cfg.public_addr, &token)
	);

	Ok(())
}

async fn shutdown_signal() {
	let ctrl_c = async {
		signal::ctrl_c()
			.await
			.expect("failed to install Ctrl+C handler");
	};

	let terminate = async {
		signal::unix::signal(signal::unix::SignalKind::terminate())
			.expect("failed to install signal handler")
			.recv()
			.await;
	};

	tokio::select! {
		_ = ctrl_c => {},
		_ = terminate => {},
	}
}
