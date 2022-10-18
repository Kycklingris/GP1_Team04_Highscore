use std::sync::RwLock;

use actix_web::{body::BoxBody, http::header::ContentType, HttpRequest, HttpResponse, Responder};

use rusqlite::{Connection, Result};

use crate::highscore::*;
use serde::Serialize;

pub struct AppState {
	pub tmp: RwLock<u32>,
}

impl Serialize for AppState {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let highscores = self.get_scores();

		serializer.collect_seq(highscores.iter())
	}
}

impl Responder for AppState {
	type Body = BoxBody;

	fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
		(&self).respond_to(req)
	}
}

impl Responder for &AppState {
	type Body = BoxBody;

	fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
		let res_body = serde_json::to_string(&self).unwrap();

		HttpResponse::Ok()
			.content_type(ContentType::json())
			.body(res_body)
	}
}

impl AppState {
	pub fn load() -> Self {
		let mut path = std::env::current_exe().expect("Unable to get the current exe path");
		path.pop();
		path.push("state");

		std::fs::create_dir_all(path).expect("Unable to create the directory");

		let conn =
			Connection::open("./state/highscores.sqlite3").expect("Unable to open the database");

		conn.execute(
			"CREATE TABLE if not exists highscores (
				version TEXT NOT NULL,
				score 	INTEGER NOT NULL,
				name 	TEXT NOT NULL	
			)",
			(),
		)
		.expect("unable to create db table");

		Self {
			tmp: RwLock::new(1),
		}
	}

	pub fn get_scores(&self) -> Highscores {
		let tmp = self.tmp.read().expect("unable to lock the rwlock");

		let mut highscores: Vec<Highscore> = Vec::new();
		let conn =
			Connection::open("./state/highscores.sqlite3").expect("Unable to read the database");
		let mut stmt = conn
			.prepare("SELECT version, score, name FROM highscores")
			.expect("Unable to read the database");
		let highscores_iter = stmt
			.query_map([], |row| {
				let highscore = Highscore {
					version: row.get(0)?,
					score: row.get(1)?,
					name: row.get(2)?,
				};
				Ok(highscore)
			})
			.expect("Unable to map the database");

		for row in highscores_iter {
			match row {
				Ok(r) => highscores.push(r),
				Err(_) => {}
			}
		}

		if *tmp == 1 {
			drop(tmp)
		}

		highscores.into()
	}

	pub fn get_versioned_scores(&self, search_version: String) -> Highscores {
		let tmp = self.tmp.read().expect("unable to lock the rwlock");
		let mut highscores: Vec<Highscore> = Vec::new();
		let conn =
			Connection::open("./state/highscores.sqlite3").expect("Unable to read the database");
		let mut stmt = conn
			.prepare("SELECT version, score, name FROM highscores WHERE version=:version; ")
			.expect("Unable to read the database");
		let highscores_iter = stmt
			.query_map(&[(":version", search_version.as_str())], |row| {
				let highscore = Highscore {
					version: row.get(0)?,
					score: row.get(1)?,
					name: row.get(2)?,
				};
				Ok(highscore)
			})
			.expect("Unable to map the database");

		for row in highscores_iter {
			match row {
				Ok(r) => highscores.push(r),
				Err(_) => {}
			}
		}

		if *tmp == 1 {
			drop(tmp)
		}

		highscores.into()
	}

	pub fn get_top_ten(&self, search_version: String) -> Highscores {
		let tmp = self.tmp.read().expect("unable to lock the rwlock");
		let mut highscores: Vec<Highscore> = Vec::new();
		let conn =
			Connection::open("./state/highscores.sqlite3").expect("Unable to read the database");
		let mut stmt = conn
			.prepare("SELECT version, score, name FROM highscores WHERE version=:version ORDER BY score DESC")
			.expect("Unable to read the database");
		let highscores_iter = stmt
			.query_map(&[(":version", search_version.as_str())], |row| {
				let highscore = Highscore {
					version: row.get(0)?,
					score: row.get(1)?,
					name: row.get(2)?,
				};
				Ok(highscore)
			})
			.expect("Unable to map the database");

		for row in highscores_iter {
			match row {
				Ok(r) => highscores.push(r),
				Err(_) => {}
			}
		}
		if *tmp == 1 {
			drop(tmp)
		}

		highscores.into()
	}
}
