 use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
	// Arrange
	let app = spawn_app().await;
	let body = "name=jamie&email=hello%40madebyjamie.com";

	// Act
	let response = app.post_subscriptions(body.into()).await;

	// Assert
	assert_eq!(200, response.status().as_u16());

	let saved = sqlx::query!("SELECT email, name FROM subscriptions")
		.fetch_one(&app.db_pool)
		.await
		.expect("Failed to fetch saved subscription");

	assert_eq!(saved.email, "hello@madebyjamie.com");
	assert_eq!(saved.name, "jamie");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_empty() {
	// Arrange
	let app = spawn_app().await;
	let test_cases = vec![
		("name=&email=hello%40madebyjamie.com", "empty name"),
		("name=jamie&email=", "empty email"),
		("name=jamie&email=not-an-email", "invalid email"),
	];

	// Act

	for (body, description) in test_cases {
		// Act
		let response = app.post_subscriptions(body.into()).await;

		// Assert
		assert_eq!(
			400,
			response.status().as_u16(),
			// Additional custom error messages
			"The API did not return a 400 Bad Request when the payload was {}.",
			description
		);
	}
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
	// Arrange
	let app = spawn_app().await;
	let test_cases = vec![
		("name=jamie", "missing email"),
		("email=hello%40madebyjamie.com", "missing name"),
		("", "missing name and email"),
	];

	for (invalid_body, error_message) in test_cases {
		// Act
		let response = app.post_subscriptions(invalid_body.into()).await;

		// Assert
		assert_eq!(
			400,
			response.status().as_u16(),
			// Additional custom error messages
			"The API did not fail with a 400 Bad Request when the payload was {}.",
			error_message
		);
	}
}