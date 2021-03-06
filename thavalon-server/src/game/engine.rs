//! THavalon game engine, implemented as an async task. This starts a `GameState` state machine and runs it to completion.

use futures::future::{self, FutureExt};
use tokio::time;

use super::interactions::Interactions;
use super::messages::GameError;
use super::Game;

use super::state::{Effect, GameStateWrapper};

/// Runs a THavalon game to completion.
pub async fn run_game<I: Interactions>(game: Game, interactions: &mut I) -> Result<(), GameError> {
    let (mut state, initial_effects) = GameStateWrapper::new(game);
    for effect in initial_effects {
        match effect {
            Effect::Broadcast(message) => {
                if let Err(e) = interactions.send(message).await {
                    log::error!("Error broadcasting message: {}", e);
                }
            }
            Effect::Send(player, message) => {
                if let Err(e) = interactions.send_to(&player, message).await {
                    log::error!("Error sending message to {}: {}", player, e);
                }
            }
            _ => panic!("Unexpected initial effect {:?}", effect),
        }
    }

    // At some points in the game, players have a certain time window to do something in. Using an
    // Either<Pending, Delay> means we can always use select below, without having to worry about whether or not there's
    // an active timeout.
    let mut timeout = future::pending().left_future();

    while !state.is_done() {
        let ((next_state, effects), player) = tokio::select! {
            _ = &mut timeout => {
                // Once the timeout future completes, we should reset it to the pending future. Otherwise, we'd keep
                // polling the time::delay_for future after it's completed, which isn't necessarily supported.
                timeout = future::pending().left_future();
                (state.handle_timeout(), None)
            },
            msg = interactions.receive() => match msg {
                Ok((player, action)) => (state.handle_action(&player, action), Some(player)),
                Err(e) => {
                    log::error!("Could not receive player input: {}", e);
                    continue;
                }
            }
        };

        for effect in effects {
            match effect {
                Effect::Broadcast(message) => {
                    if let Err(e) = interactions.send(message).await {
                        log::error!("Error broadcasting message: {}", e);
                    }
                }
                Effect::Send(player, message) => {
                    if let Err(e) = interactions.send_to(&player, message).await {
                        log::error!("Error sending message to {}: {}", player, e);
                    }
                }
                Effect::Reply(message) => {
                    // player is only None if the timeout fired, and handle_timeout() should never return an
                    // Effect::Reply because there's no player to reply to.
                    let player = player
                        .as_ref()
                        .expect("handle_timeout() returned an Effect::Reply");
                    if let Err(e) = interactions.send_to(player, message).await {
                        log::error!("Error sending message to {}: {}", player, e);
                    }
                }
                Effect::StartTimeout(duration) => {
                    timeout = time::delay_for(duration).right_future();
                }
                Effect::ClearTimeout => timeout = future::pending().left_future(),
                Effect::Send(receiving_player, message) => {
                    interactions.send_to(&receiving_player, message).await;
                }
            }
        }
        state = next_state;
    }

    Ok(())
}
