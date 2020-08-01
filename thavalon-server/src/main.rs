#![allow(dead_code)]

use fern::colors::{Color, ColoredLevelConfig};
use tokio::stream::StreamExt;

mod connections;
mod game;
mod lobbies;

use self::game::{GameRunner, ControlRequest};

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

    let (mut game_tx, mut game_rx) = GameRunner::spawn();
    let players = vec![
        (10, "Ben".to_string()),
        (20, "Paul".to_string()),
        (30, "Jared".to_string()),
        (40, "Andrew".to_string()),
        (50, "Galen".to_string())
    ];
    let mut responses = vec![];
    for (id, name) in players.into_iter() {
        game_tx.send(ControlRequest::AddPlayer { id, name }).await.unwrap();
        responses.push(game_rx.next().await.unwrap());
    }
    game_tx.send(ControlRequest::StartGame).await.unwrap();
    game_rx.next().await.unwrap();
    
    connections::serve_connections().await;
}
