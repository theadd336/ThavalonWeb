use fern::colors::{Color, ColoredLevelConfig};

mod game;

use self::game::GameRunner;

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
        .level_for("warp", log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}

#[tokio::main]
async fn main() {
    setup_logger().expect("Could not set up logging");

    let channels = GameRunner::launch(vec![
        (10, "Ben".to_string()),
        (20, "Paul".to_string()),
        (30, "Jared".to_string()),
        (40, "Andrew".to_string()),
        (50, "Galen".to_string())
    ]);
    // do stuff with channels

    println!("Hello, world!");
}
