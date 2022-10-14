use std::{
	sync::RwLock,
};

use actix_web::{
	body::BoxBody, get, http::header::ContentType, post, web, App, HttpRequest,
	HttpResponse, HttpServer, Responder,
};

use rusqlite::{Connection, Result};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Highscore {
	pub score: u32,
	pub name: String,
	pub version: String,
}

struct Highscores {
	a: Vec<Highscore>,
}

impl core::ops::Deref for Highscores {
	type Target = Vec<Highscore>;

	fn deref(&self) -> &Self::Target {
		&self.a
	}
}

impl core::ops::DerefMut for Highscores {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.a
	}
}

impl core::convert::From<Vec<Highscore>> for Highscores {
	fn from(original: Vec<Highscore>) -> Self {
		Self { a: original }
	}
}

impl Responder for Highscore {
	type Body = BoxBody;

	fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
		let res_body = serde_json::to_string(&self).unwrap();

		HttpResponse::Ok()
			.content_type(ContentType::json())
			.body(res_body)
	}
}

impl Serialize for Highscores {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.collect_seq(self.a.iter())
	}
}

struct AppState {
	tmp: RwLock<u32>,
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

#[get("/highscores")]
async fn get_highscores(req: actix_web::HttpRequest, data: web::Data<AppState>) -> impl Responder {
	data.get_ref().respond_to(&req)
}

#[get("/highscores/{version}")]
async fn get_highscores_version(
	version: web::Path<String>,
	_: actix_web::HttpRequest,
	data: web::Data<AppState>,
) -> impl Responder {
	let highscores = data.get_versioned_scores(version.to_string());
	let res_body = serde_json::to_string(&highscores).unwrap();

	HttpResponse::Ok()
		.content_type(ContentType::json())
		.body(res_body)
}

#[get("/top_ten/{version}")]
async fn get_top_ten(
	version: web::Path<String>,
	_: actix_web::HttpRequest,
	data: web::Data<AppState>,
) -> impl Responder {
	let highscores = data.get_top_ten(version.to_string());
	let res_body = serde_json::to_string(&highscores[0..10]).unwrap();

	HttpResponse::Ok()
		.content_type(ContentType::json())
		.body(res_body)
}

#[post("/highscore")]
async fn set_highscore(req: web::Json<Highscore>, data: web::Data<AppState>) -> impl Responder {
	let mut rwlock = data.tmp.write().expect("lock is poisoned");
	let conn = Connection::open("./state/highscores.sqlite3").expect("Unable to read the database");

	let highscore = req.0.to_owned();

	conn.execute(
		"INSERT INTO highscores (score, version, name) VALUES (?1, ?2, ?3)",
		(&highscore.score, &highscore.version, &highscore.name),
	)
	.expect("Unable to add row to database");

	*rwlock = 1;
	HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let mut path = std::env::current_exe()?;
	path.pop();
	std::env::set_current_dir(path)?;

	let state = web::Data::new(AppState::load());

	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::clone(&state))
			.service(get_highscores)
			.service(set_highscore)
			.service(get_highscores_version)
			.service(get_top_ten)
	})
	.bind(("0.0.0.0", 80))?
	.bind(("0.0.0.0", 443))?
	.run()
	.await
}
