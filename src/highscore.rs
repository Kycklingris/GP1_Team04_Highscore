use actix_web::{body::BoxBody, http::header::ContentType, HttpRequest, HttpResponse, Responder};

use rusqlite::Result;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Highscore {
	pub score: u32,
	pub name: String,
	pub version: String,
}

pub struct Highscores {
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
