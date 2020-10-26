//! Main entry point into ThavalonWeb's backend API

use fern::colors::{Color, ColoredLevelConfig};

mod connections;
mod database;
mod game;
mod notifications;

fn setup_logger() -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new()
        .info(Color::Green)
        .debug(Color::Cyan);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .level_for("hyper", log::LevelFilter::Info)
        .level_for("warp", log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}

#[tokio::main]
async fn main() {
    setup_logger().expect("Could not set up logging");
    database::initialize_mongo_client().await;
    connections::serve_connections().await;
}
