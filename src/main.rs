use actix_web::{
	get, http::header::ContentType, post, web, App, HttpResponse, HttpServer, Responder,
};

use rusqlite::Connection;

mod highscore;
use highscore::*;

mod state;
use state::*;

#[get("/highscores")]
async fn get_highscores(req: actix_web::HttpRequest, data: web::Data<AppState>) -> impl Responder {
	let highscores = data.get_scores();
	let res_body = serde_json::to_string(&highscores[0..50]).unwrap();

	HttpResponse::Ok()
		.content_type(ContentType::json())
		.body(res_body)
}

#[get("/highscores/{version}")]
async fn get_highscores_version(
	version: web::Path<String>,
	_: actix_web::HttpRequest,
	data: web::Data<AppState>,
) -> impl Responder {
	let highscores = data.get_versioned_scores(version.to_string());
	let res_body = serde_json::to_string(&highscores[0..50]).unwrap();

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
