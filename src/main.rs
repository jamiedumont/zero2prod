use zero2prod::startup::Application;
use zero2prod::config::get_configuration;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Panic if we can't find our config file
    let config = get_configuration().expect("Failed to read config");

    let application = Application::build(config).await?;
    application.run_until_stopped().await?;
    Ok(())
}
