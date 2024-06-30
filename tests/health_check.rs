use std::net::TcpListener;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
	// Arrange
	let (app_address, client) = setup_for_tests();
	
	// Act
	let body = "name=jamie&email=hello%40madebyjamie.com";
	let response = client
		.post(&format!("{}/subscriptions", &app_address))
		.header("Content-Type", "application/x-www-form-urlencoded")
		.body(body)
		.send()
		.await
		.expect("Failed to execute request");
	
	// Assert
	assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
	// Arrange
	let (app_address, client) = setup_for_tests();
	let test_cases = vec![
		("name=jamie", "missing email"),
		("email=hello%40madebyjamie.com", "missing name"),
		("", "missing name and email")
	];
	
	for (invalid_body, error_message) in test_cases {
		// Act
		let response = client
			.post(&format!("{}/subscriptions", &app_address))
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
}

#[tokio::test]
async fn health_check_works() {
	// Arrange
	let (server_address, client) = setup_for_tests();
	
	// Act
	let response = client
		.get(&format!("{}/health-check", server_address))
		.send()
		.await
		.expect("Failed to execute request.");
	
	// Assert
	assert!(response.status().is_success());
	assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
	let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
	let port = listener.local_addr().unwrap().port();
	let server = zero2prod::startup::run(listener).expect("Failed to bind address");
	let _ = tokio::spawn(server);
	
	format!("http://127.0.0.1:{}", port)
}

fn setup_for_tests() -> (String, reqwest::Client) {
	let app_address = spawn_app();
	let client = reqwest::Client::new();
	(app_address, client)
}