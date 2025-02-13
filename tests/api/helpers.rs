use once_cell::sync::Lazy;
use secrecy::Secret;
use sqlx::{PgPool, Executor, PgConnection, Connection};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::config::{get_configuration, DatabaseSettings};
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::startup::{Application, get_connection_pool};

static TRACING: Lazy<()> = Lazy::new(|| {
	let default_filter_level = "info".to_string();
	let subscriber_name = "test".to_string();

	if std::env::var("TEST_LOG").is_ok() {
		let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
		init_subscriber(subscriber);
	} else {
		let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
		init_subscriber(subscriber);
	}
});

pub struct TestApp {
	pub address: String,
	pub db_pool: PgPool,
	pub email_server: MockServer,
}

impl TestApp {
	pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
		reqwest::Client::new()
			.post(&format!("{}/subscriptions", &self.address))
			.header("Content-Type", "application/x-www-form-urlencoded")
			.body(body)
			.send()
			.await
			.expect("Failed to execute request")
	}
}

pub async fn spawn_app() -> TestApp {
	// First time 'initialise' is invoked the code in 'TRACING' is executed.
	// All other invocations will instead skip execution
	Lazy::force(&TRACING);

	// Launch a mock server to stand in for Postmark
	let email_server = MockServer::start().await;

	// Randomise config to ensure test isolation
	let configuration = {
		let mut c = get_configuration().expect("Failed to read config");
		c.database.database_name = Uuid::new_v4().to_string();
		c.application.port = 0;
		c.email_client.base_url = email_server.uri();
		c
	};

	// Create and migrate database
	configure_database(&configuration.database).await;

	let application = Application::build(configuration.clone())
		.await
		.expect("Failed to build application");

	let address = format!("http://127.0.0.1:{}", &application.port());
	let _ = tokio::spawn(application.run_until_stopped());

	TestApp {
		address,
		db_pool: get_connection_pool(&configuration.database),
		email_server
	}
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
	// Create database
	let maintenance_settings = DatabaseSettings {
		database_name: "postgres".to_string(),
		username: "postgres".to_string(),
		password: Secret::new("password".to_string()),
		..config.clone()
	};
	let mut connection = PgConnection::connect_with(&maintenance_settings.connect_options())
		.await
		.expect("Failed to connect to Postgres");
	connection
		.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
		.await
		.expect("Failed to create database.");

	// Migrate database
	let connection_pool = PgPool::connect_with(config.connect_options())
		.await
		.expect("Failed to connect to Postgres.");
	sqlx::migrate!("./migrations")
		.run(&connection_pool)
		.await
		.expect("Failed to migrate the database");
	connection_pool
}