use zero2prod::startup::run;
use zero2prod::config::get_configuration;
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Panic if we can't find our config file
    let config = get_configuration().expect("Failed to read config");
    let connection_pool = PgPool::connect(
            &config.database.connection_string()
        )
        .await
        .expect("Failed to connect to Postgres");
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
