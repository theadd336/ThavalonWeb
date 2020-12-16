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

    /// Receive the next message from any player
    async fn receive(&mut self) -> Result<(String, Action), GameError>;
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

    pub fn remove_player(&mut self, name: &String) {
        self.inbox.remove(name);
        self.outbox.remove(name);
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
        let sends = self.outbox.iter_mut().map(|(_name, sender)| {
            sender
                .send(message.clone())
                .map_err(move |_| GameError::PlayerDisconnected)
        });
        future::join_all(sends).await.into_iter().collect()
    }

    async fn receive(&mut self) -> Result<(String, Action), GameError> {
        match self.inbox.next().await {
            Some(msg) => Ok(msg),
            None => Err(GameError::PlayerDisconnected),
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

        async fn receive(&mut self) -> Result<(String, Action), GameError> {
            match self.actions.pop_front() {
                Some(msg) => Ok(msg),
                None => Err(GameError::PlayerDisconnected),
            }
        }
    }
}
