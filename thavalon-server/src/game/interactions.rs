//! Defines how players interact with the game state.

use std::collections::HashMap;

use async_trait::async_trait;
use futures::{future, TryFutureExt};
use tokio::stream::{StreamExt, StreamMap};
use tokio::sync::mpsc;

use super::messages::{Action, GameError, Message};

#[async_trait]
pub trait Interactions {
    /// Send a message to a specific player
    async fn send_to(&mut self, player: &str, message: Message) -> Result<(), GameError>;

    /// Send a message to all players
    async fn send(&mut self, message: Message) -> Result<(), GameError>;

    /// Receives messages from players until one satisfies the given function. The function can either return `Ok`, in which case the
    /// result is immediately returned, or `Err`, in which case the error message is sent to the player and this waits for the next message.
    ///
    /// Think of this like a one-off `filter_map` operation.
    async fn receive<F, R>(&mut self, mut f: F) -> Result<R, GameError>
    where
        R: Send,
        F: FnMut(String, Action) -> Result<R, String> + Send;
}

/// An Interactions that uses per-player MPSC channels
pub struct ChannelInteractions {
    inbox: StreamMap<String, mpsc::Receiver<Action>>,
    outbox: HashMap<String, mpsc::Sender<Message>>,
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
        name: String,
        incoming: mpsc::Receiver<Action>,
        outgoing: mpsc::Sender<Message>,
    ) {
        self.inbox.insert(name.clone(), incoming);
        self.outbox.insert(name, outgoing);
    }
}

#[async_trait]
impl Interactions for ChannelInteractions {
    async fn send_to(&mut self, player: &str, message: Message) -> Result<(), GameError> {
        self.outbox
            .get_mut(player)
            .unwrap()
            .send(message)
            .await
            .map_err(|_| GameError::PlayerDisconnected)
    }

    async fn send(&mut self, message: Message) -> Result<(), GameError> {
        let sends = self.outbox.iter_mut().map(|(name, sender)| {
            sender
                .send(message.clone())
                .map_err(move |_| GameError::PlayerDisconnected)
        });
        future::join_all(sends).await.into_iter().collect()
    }

    async fn receive<F, R>(&mut self, mut f: F) -> Result<R, GameError>
    where
        R: Send,
        F: FnMut(String, Action) -> Result<R, String> + Send,
    {
        loop {
            match self.inbox.next().await {
                Some((player, action)) => {
                    let out = self.outbox.get_mut(&player).unwrap();
                    match f(player, action) {
                        Ok(result) => return Ok(result),
                        Err(msg) => {
                            out.send(Message::Error(msg))
                                .map_err(|_| GameError::PlayerDisconnected)
                                .await?
                        }
                    }
                }
                None => return Err(GameError::PlayerDisconnected),
            }
        }
    }
}

#[cfg(test)]
pub(super) mod test {
    use std::collections::VecDeque;

    use async_trait::async_trait;

    use super::super::messages::{Action, GameError, Message};
    use super::Interactions;

    pub struct TestInteractions {
        broadcasts: Vec<Message>,
        messages: Vec<(String, Message)>,
        actions: VecDeque<(String, Action)>,
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

        pub fn push_action(&mut self, player: String, action: Action) {
            self.actions.push_back((player, action));
        }

        pub fn extend_actions<I: IntoIterator<Item = (String, Action)>>(&mut self, iter: I) {
            self.actions.extend(iter);
        }

        pub fn broadcasts(&self) -> &[Message] {
            self.broadcasts.as_slice()
        }

        pub fn messages(&self) -> &[(String, Message)] {
            self.messages.as_slice()
        }

        pub fn messages_for(&self, player: String) -> impl Iterator<Item = &Message> {
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

    #[async_trait]
    impl Interactions for TestInteractions {
        async fn send_to(&mut self, player: &str, message: Message) -> Result<(), GameError> {
            self.messages.push((player.to_string(), message));
            Ok(())
        }

        async fn send(&mut self, message: Message) -> Result<(), GameError> {
            self.broadcasts.push(message);
            Ok(())
        }

        async fn receive<F, R>(&mut self, mut f: F) -> Result<R, GameError>
        where
            R: Send,
            F: FnMut(String, Action) -> Result<R, String> + Send,
        {
            loop {
                match self.actions.pop_front() {
                    Some((player, action)) => match f(player.clone(), action) {
                        Ok(result) => return Ok(result),
                        Err(msg) => self.messages.push((player, Message::Error(msg))),
                    },
                    None => return Err(GameError::PlayerDisconnected),
                }
            }
        }
    }
}
