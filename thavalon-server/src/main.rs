use fern::colors::{ColoredLevelConfig, Color};

mod game;

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

    let game = game::Game::roll(vec!["Ben".to_string(), "Paul".to_string(), "Jared".to_string(), "Andrew".to_string(), "Galen".to_string()]);
    let (mut runner, _) = game::runner::GameRunner::new(game);
    tokio::spawn(async move {
        runner.run().await;
    }).await.unwrap();

    println!("Hello, world!");
}
