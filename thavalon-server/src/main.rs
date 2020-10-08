#![allow(dead_code)]

use fern::colors::{Color, ColoredLevelConfig};
use tokio::stream::StreamExt;

mod connections;
mod database;
mod game;
mod lobbies;

use self::game::{ControlRequest, GameRunner};

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
    connections::serve_connections().await;
}
