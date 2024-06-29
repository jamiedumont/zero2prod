use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_web::dev::Server;
use std::net::TcpListener;

async fn health_check() -> HttpResponse {
	// Return 200 response
	HttpResponse::Ok().finish()
}

#[derive(serde::Deserialize)]
struct FormData {
	email: String,
	name: String
}

async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
	HttpResponse::Ok().finish()
}


pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
	let server = HttpServer::new(|| {
		App::new()
		.route("/health-check", web::get().to(health_check))
		.route("/subscriptions", web::post().to(subscribe))
	})
	.listen(listener)?
	.run();
	
	Ok(server)
}