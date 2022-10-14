use std::sync::Mutex;

use actix_web::{
	body::BoxBody, get, http::header::ContentType, post, put, web, App, Error, HttpRequest,
	HttpResponse, HttpServer, Responder,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Highscore {
	pub score: u32,
	pub name: String,
	pub version: String,
}

impl Responder for Highscore {
	type Body = BoxBody;

	fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
		let res_body = serde_json::to_string(&self).unwrap();

		HttpResponse::Ok()
			.content_type(ContentType::json())
			.body(res_body)
	}
}

struct AppState {
	pub highscores: Mutex<Vec<Highscore>>,
	pub top_ten: Mutex<[Highscore; 10]>
}

impl Serialize for AppState {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.collect_seq(self.highscores.lock().expect("").iter())
	}
}

impl Responder for AppState {
	type Body = BoxBody;

	fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
		(&self).respond_to(req)
	}
}

impl AppState {
	pub fn new() -> Self {
		let mut highscores = Mutex::new(Vec::new());

		highscores.get_mut().unwrap().push(Highscore { score: 19, name: "abow".to_string(), version: "0.0.1".to_string() });

		Self { highscores }
	}
}

impl Responder for &AppState {
	type Body = BoxBody;

	fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
    match req {
       
      _ => {}
    }	
	
		let res_body = serde_json::to_string(&self).unwrap();

		HttpResponse::Ok()
			.content_type(ContentType::json())
			.body(res_body)
	}
}

#[get("/highscores")]
async fn get_highscores(req: actix_web::HttpRequest, data: web::Data<AppState>) -> impl Responder {
	data.get_ref().respond_to(&req)
}

#[get("/top_ten")]
async fn get_top_ten(req: actix_web::HttpRequest, data web::Data<AppState>) -> impl Responder {
  
}

#[post("/highscore")]
async fn set_highscore(req: web::Json<Highscore>, data: web::Data<AppState>) -> impl Responder {
	let highscore = req.0;
	
	let mut state = data.highscores.lock().expect("Unable to get mutable access to the state");

	state.push(highscore);

	HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let mut path = std::env::current_exe()?;
	path.pop();
	std::env::set_current_dir(path)?;

	let state = web::Data::new(AppState::new());

	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::clone(&state))
			.service(get_highscores)
			.service(set_highscore)
			.service(get_top_ten)
	})
	.bind(("0.0.0.0", 80))?
	.bind(("0.0.0.0", 443))?
	.run()
	.await
}
