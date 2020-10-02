use fern::colors::{Color, ColoredLevelConfig};
mod database;
mod rest;
mod validation;

/// Main entry point
#[tokio::main]
async fn main() {
    setup_logger().expect("Fatal error while setting up logging.");
    rest::accept_requests().await;
}

// Sets up the logger for pretty colors.
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
        .level_for("hyper", log::LevelFilter::Warn)
        .level_for("warp", log::LevelFilter::Warn)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}
