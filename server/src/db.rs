//! Helpers the modules share for the database

use pg::{Database, FromRow, UniqueId};

/// Runs the migrations of a module that didn't run yet, in order
pub async fn migrate(db: &Database, migrations: &[(&str, &str)]) {
	let m = db.migrations();
	let mut conn = db.get().await.expect("no database connection");

	for (name, sql) in migrations {
		m.add(&mut conn, name, sql)
			.await
			.expect("failed to run migration");
	}
}

#[derive(FromRow)]
pub struct IdRow {
	pub id: UniqueId,
}
