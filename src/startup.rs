use crate::config::{Settings, DatabaseSettings};
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};

use actix_web::{web, App, HttpServer};
use actix_web::dev::Server;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
	port: u16,
	server: Server
}

impl Application {
	pub async fn build(config: Settings) -> Result<Self, std::io::Error> {
		let connection_pool = get_connection_pool(&config.database);

		let sender_email = config
			.email_client
			.sender()
        	.expect("Invalid sender email address");

    	let timeout = config.email_client.timeout();
    	let email_client = EmailClient::new(
        	config.email_client.base_url,
        	sender_email,
        	config.email_client.authorization_token,
        	timeout
    	);
    	let address = format!("{}:{}", config.application.host, config.application.port);
    	let listener = TcpListener::bind(address)?;
		let port = listener.local_addr().unwrap().port();

    	let server = run(listener, connection_pool, email_client)?;
		Ok(Self {port, server})
	}

	pub fn port(&self) -> u16 {
		self.port
	}

	pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
		self.server.await
	}
}

pub fn get_connection_pool(
	config: &DatabaseSettings
) -> PgPool {
	PgPoolOptions::new().connect_lazy_with(config.connect_options())
}

pub fn run(
	listener: TcpListener,
	db_pool: PgPool,
	email_client: EmailClient
) -> Result<Server, std::io::Error> {
	let db_pool = web::Data::new(db_pool);
	let email_client = web::Data::new(email_client);
	let server = HttpServer::new(move || {
		App::new()
		.wrap(TracingLogger::default())
		.route("/health-check", web::get().to(health_check))
		.route("/subscriptions", web::post().to(subscribe))
		.app_data(db_pool.clone())
		.app_data(email_client.clone())
	})
	.listen(listener)?
	.run();

	Ok(server)
}
