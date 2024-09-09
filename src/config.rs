use secrecy::{Secret, ExposeSecret};
use crate::domain::SubscriberEmail;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
	pub database: DatabaseSettings,
	pub application: ApplicationSettings,
	pub email_client: EmailClientSettings
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
	#[serde(deserialize_with = "deserialize_number_from_string")]
	pub port: u16,
	pub host: String
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
	pub username: String,
	pub password: Secret<String>,
	#[serde(deserialize_with = "deserialize_number_from_string")]
	pub port: u16,
	pub host: String,
	pub database_name: String,
	pub require_ssl: bool,
}

impl DatabaseSettings {
	pub fn connect_options(&self) -> PgConnectOptions {
		let ssl_mode = if self.require_ssl {
			PgSslMode::Require
		} else {
			PgSslMode::Prefer
		};
		PgConnectOptions::new()
			.host(&self.host)
			.username(&self.username)
			.password(self.password.expose_secret())
			.port(self.port)
			.ssl_mode(ssl_mode)
			.database(&self.database_name)
	}
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailClientSettings {
	pub base_url: String,
	pub sender_email: String,
	pub authorization_token: Secret<String>,
	pub timeout_milliseconds: u64
}

impl EmailClientSettings {
	pub fn sender(&self) -> Result<SubscriberEmail, String> {
		SubscriberEmail::parse(self.sender_email.clone())
	}

	pub fn timeout(&self) -> std::time::Duration {
		std::time::Duration::from_millis(self.timeout_milliseconds)
	}
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
	let base_path = std::env::current_dir().expect("Failed to determine the current directory");
	let configuration_directory = base_path.join("configuration");

	// Init config reader
	let settings = config::Config::builder()
		.add_source(config::File::from(
			configuration_directory.join("base.toml")
		))
		.build()?;

	// Try to convert what we read into our Settings struct
	settings.try_deserialize::<Settings>()
}