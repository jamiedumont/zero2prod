use std::net::TcpListener;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use zero2prod::config::{get_configuration, DatabaseSettings};
use uuid::Uuid;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
	// Arrange
	let (app, client) = setup_for_tests().await;
	
	// Act
	let body = "name=jamie&email=hello%40madebyjamie.com";
	let response = client
		.post(&format!("{}/subscriptions", &app.address))
		.header("Content-Type", "application/x-www-form-urlencoded")
		.body(body)
		.send()
		.await
		.expect("Failed to execute request");
	
	// Assert
	assert_eq!(200, response.status().as_u16());
	
	let saved = sqlx::query!("SELECT email, name FROM subscriptions")
		.fetch_one(&app.db_pool)
		.await
		.expect("Failed to fetch saved subscription");
		
	assert_eq!(saved.email, "hello@madebyjamie.com");
	assert_eq!(saved.name, "jamie");
	
	cleanup_database(&app).await;
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
	// Arrange
	let (test_app, client) = setup_for_tests().await;
	let test_cases = vec![
		("name=jamie", "missing email"),
		("email=hello%40madebyjamie.com", "missing name"),
		("", "missing name and email")
	];
	
	for (invalid_body, error_message) in test_cases {
		// Act
		let response = client
			.post(&format!("{}/subscriptions", &test_app.address))
			.header("Content-Type", "application/x-www-form-urlencoded")
			.body(invalid_body)
			.send()
			.await
			.expect("Failed to execute request");
			
		// Assert
		assert_eq!(
			400,
			response.status().as_u16(),
			// Additional custom error messages
			"The API did not fail with a 400 Bad Request when the payload was {}.", error_message
		);
	}
	
	cleanup_database(&test_app).await;
}

#[tokio::test]
async fn health_check_works() {
	// Arrange
	let (test_app, client) = setup_for_tests().await;
	
	// Act
	let response = client
		.get(&format!("{}/health-check", test_app.address))
		.send()
		.await
		.expect("Failed to execute request.");
	
	// Assert
	assert!(response.status().is_success());
	assert_eq!(Some(0), response.content_length());
	
	cleanup_database(&test_app).await;
}

pub struct TestApp {
	pub address: String,
	pub db_pool: PgPool,
	pub db_config: DatabaseSettings
}

async fn spawn_app() -> TestApp {
	let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
	let port = listener.local_addr().unwrap().port();
	let address = format!("http://127.0.0.1:{}", port);
	
	let mut configuration = get_configuration().expect("Failed to read config");
	configuration.database.database_name = Uuid::new_v4().to_string();
	let conn_pool = configure_database(&configuration.database).await;
		
	let server = zero2prod::startup::run(listener, conn_pool.clone()).expect("Failed to bind address");
	let _ = tokio::spawn(server);
	
	TestApp {
		address,
		db_pool: conn_pool,
		db_config: configuration.database
	}
}

pub async fn cleanup_database(app: &TestApp) {
	app.db_pool.close().await;
	
	let mut connection = PgConnection::connect(
			&app.db_config.connection_string_without_db()
		)
		.await
		.expect("Failed to connect to Postgres");
	connection
		.execute(format!(r#"DROP DATABASE "{}";"#, app.db_config.database_name).as_str())
		.await
		.expect("Failed to drop database");
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
	// Create database
	let mut connection = PgConnection::connect(
			&config.connection_string_without_db()
		)
		.await
		.expect("Failed to connect to Postgres");
	connection
		.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
		.await
		.expect("Failed to create database");
	
	// MIgrate database
	let connection_pool = PgPool::connect(&config.connection_string())
		.await
		.expect("Failed to connect to Postgres");
	
	sqlx::migrate!("./migrations")
		.run(&connection_pool)
		.await
		.expect("Failed to migrate the database");
	
	connection_pool	
}

async fn setup_for_tests() -> (TestApp, reqwest::Client) {
	let test_app = spawn_app().await;
	let client = reqwest::Client::new();
	(test_app, client)
}