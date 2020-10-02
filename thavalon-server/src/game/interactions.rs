//! Defines how players interact with the game state.

use std::collections::HashMap;

use async_trait::async_trait;
use futures::{future, TryFutureExt};
use tokio::stream::{StreamExt, StreamMap};
use tokio::sync::mpsc;

use super::runner::{Action, GameError, Message};
use super::PlayerId;

#[async_trait(?Send)]
pub(super) trait Interactions {
    /// Send a message to a specific player
    async fn send_to(&mut self, player: PlayerId, message: Message) -> Result<(), GameError>;

    /// Send a message to all players
    async fn send(&mut self, message: Message) -> Result<(), GameError>;

    /// Receives messages from players until one satisfies the given function. The function can either return `Ok`, in which case the
    /// result is immediately returned, or `Err`, in which case the error message is sent to the player and this waits for the next message.
    ///
    /// Think of this like a one-off `filter_map` operation.
    async fn receive<F, R>(&mut self, mut f: F) -> Result<R, GameError>
    where
        F: FnMut(PlayerId, Action) -> Result<R, String>;
}

/// An Interactions that uses per-player MPSC channels
pub(super) struct ChannelInteractions {
    inbox: StreamMap<PlayerId, mpsc::Receiver<Action>>,
    outbox: HashMap<PlayerId, mpsc::Sender<Message>>,
}

impl ChannelInteractions {
    pub fn new() -> ChannelInteractions {
        ChannelInteractions {
            inbox: StreamMap::new(),
            outbox: HashMap::new(),
        }
    }

    pub fn add_player(
        &mut self,
        id: PlayerId,
        incoming: mpsc::Receiver<Action>,
        outgoing: mpsc::Sender<Message>,
    ) {
        self.inbox.insert(id, incoming);
        self.outbox.insert(id, outgoing);
    }
}

#[async_trait(?Send)]
impl Interactions for ChannelInteractions {
    async fn send_to(&mut self, player: PlayerId, message: Message) -> Result<(), GameError> {
        self.outbox
            .get_mut(&player)
            .unwrap()
            .send(message)
            .await
            .map_err(|_| GameError::PlayerUnavailable { id: player })
    }

    async fn send(&mut self, message: Message) -> Result<(), GameError> {
        let sends = self.outbox.iter_mut().map(|(id, sender)| {
            sender
                .send(message.clone())
                .map_err(move |_| GameError::PlayerUnavailable { id: *id })
        });
        future::join_all(sends).await.into_iter().collect()
    }

    async fn receive<F, R>(&mut self, mut f: F) -> Result<R, GameError>
    where
        F: FnMut(PlayerId, Action) -> Result<R, String>,
    {
        loop {
            match self.inbox.next().await {
                Some((player, action)) => match f(player, action) {
                    Ok(result) => return Ok(result),
                    Err(msg) => self.send_to(player, Message::Error(msg)).await?,
                },
                None => return Err(GameError::AllDisconnected),
            }
        }
    }
}

#[cfg(test)]
pub(super) mod test {
    use std::collections::VecDeque;

    use async_trait::async_trait;

    use super::super::runner::{Action, GameError, Message};
    use super::super::PlayerId;
    use super::Interactions;

    pub struct TestInteractions {
        broadcasts: Vec<Message>,
        messages: Vec<(PlayerId, Message)>,
        actions: VecDeque<(PlayerId, Action)>,
    }

    #[cfg(test)]
    impl TestInteractions {
        pub fn new() -> TestInteractions {
            TestInteractions {
                broadcasts: Vec::new(),
                messages: Vec::new(),
                actions: VecDeque::new(),
            }
        }

        pub fn push_action(&mut self, player: PlayerId, action: Action) {
            self.actions.push_back((player, action));
        }

        pub fn extend_actions<I: IntoIterator<Item = (PlayerId, Action)>>(&mut self, iter: I) {
            self.actions.extend(iter);
        }

        pub fn broadcasts(&self) -> &[Message] {
            self.broadcasts.as_slice()
        }

        pub fn messages(&self) -> &[(PlayerId, Message)] {
            self.messages.as_slice()
        }

        pub fn messages_for(&self, player: PlayerId) -> impl Iterator<Item = &Message> {
            self.messages
                .iter()
                .filter_map(move |(recipient, message)| {
                    if recipient == &player {
                        Some(message)
                    } else {
                        None
                    }
                })
        }
    }

    #[async_trait(?Send)]
    impl Interactions for TestInteractions {
        async fn send_to(&mut self, player: PlayerId, message: Message) -> Result<(), GameError> {
            self.messages.push((player, message));
            Ok(())
        }

        async fn send(&mut self, message: Message) -> Result<(), GameError> {
            self.broadcasts.push(message);
            Ok(())
        }

        async fn receive<F, R>(&mut self, mut f: F) -> Result<R, GameError>
        where
            F: FnMut(PlayerId, Action) -> Result<R, String>,
        {
            loop {
                match self.actions.pop_front() {
                    Some((player, action)) => match f(player, action) {
                        Ok(result) => return Ok(result),
                        Err(msg) => self.messages.push((player, Message::Error(msg))),
                    },
                    None => return Err(GameError::AllDisconnected),
                }
            }
        }
    }
}
