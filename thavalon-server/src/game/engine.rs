//! THavalon game engine, implemented as an async task.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::iter;
use std::slice;

use itertools::Itertools;
use tokio::time::{self, Duration};

use super::{Card, Game, GameSpec, PlayerId};
use super::interactions::Interactions;
use super::messages::{Action, GameError, Message};
use super::role::Role;

/// Amount of time to wait for a declaration, for declarations that have to happen within a certain timeframe.
const DECLARE_DELAY: Duration = Duration::from_secs(30);

/// Holder for shared game engine state. This captures that we generally need an immutable borrow of Game and a mutable borrow of Interactions.
struct GameEngine<'a, I: Interactions> {
    game: &'a Game,
    interactions: &'a mut I,

    /// Infinite iterator over proposal order for the game. It's... a little unfortunate we can't (yet) use impl Trait here, but if
    /// it gets hairy enough, we can always box the iterator.
    proposers: iter::Skip<iter::Cycle<iter::Cloned<slice::Iter<'a, usize>>>>,
}

/// A proposal
#[derive(Debug)]
struct Proposal<'a> {
    game: &'a Game,
    proposer: PlayerId,
    players: HashSet<PlayerId>,
}

/// The results of a vote
struct VotingResults {
    /// How each player voted
    votes: HashMap<PlayerId, bool>,
    /// Whether or not the mission was sent. A mission is sent if there are strictly more upvotes than downvotes. Ties are not sent.
    sent: bool,
}

/// Runs a THavalon game to completion.
pub async fn run_game<I: Interactions>(game: &Game, interactions: &mut I) -> Result<(), GameError> {
    let mut ge = GameEngine::new(game, interactions);

    for player in ge.game.players.iter() {
        ge.interactions
            .send_to(
                player.id,
                Message::RoleInformation {
                    role: player.role,
                    information: game.info[&player.id].clone(),
                },
            )
            .await?;
    }

    let first_proposer = ge.proposers.next().unwrap();
    let second_proposer = ge.proposers.next().unwrap();
    log::debug!(
        "{} and {} are proposing mission 1",
        game.name(first_proposer),
        game.name(second_proposer)
    );

    let first_proposal = ge.get_proposal(first_proposer, 1, 1).await?;
    let second_proposal = ge.get_proposal(second_proposer, 1, 2).await?;

    let votes = ge.vote().await?;
    let first_passed = if votes.sent {
        ge.send_mission(first_proposal, 1).await?
    } else {
        ge.send_mission(second_proposal, 1).await?
    };

    let mut failed_missions = if first_passed { 0 } else { 1 };
    let mut passed_missions = 1 - failed_missions;

    'missions: for mission in 2..=5 {
        'proposals: for proposal_num in 1..=game.spec.proposals() {
            let proposer = ge.proposers.next().unwrap();
            let proposal = ge
                .get_proposal(proposer, mission, proposal_num as u8)
                .await?;

            // If this proposal has force, skip voting
            // Otherwise, vote on the mission
            if proposal_num != game.spec.proposals() {
                let votes = ge.vote().await?;
                if !votes.sent {
                    continue 'proposals;
                }
            }

            let passed = ge.send_mission(proposal, mission as usize).await?;
            if passed {
                passed_missions += 1;
            } else {
                failed_missions += 1;
            }

            // Skip remaining missions if a team has already won (pending assasination)
            if passed_missions == 3 {
                log::debug!("3 missions have passed");
                break 'missions;
            } else if failed_missions == 3 {
                log::debug!("3 missions have failed");
                break 'missions;
            }
        }
    }

    Ok(())
}

impl<'a, I: Interactions> GameEngine<'a, I> {
    fn new(game: &'a Game, interactions: &'a mut I) -> GameEngine<'a, I> {
        let proposers = game
            .proposal_order()
            .iter()
            .cloned()
            .cycle()
            // Mission 1 is proposed by the last 2 players in the proposal order
            .skip(game.size() - 2);

        GameEngine {
            game,
            interactions,
            proposers,
        }
    }

    /// Obtain a valid proposal from the proposing player. This will continue asking them until they make a valid proposal.
    async fn get_proposal(
        &mut self,
        from: PlayerId,
        mission: u8,
        proposal: u8,
    ) -> Result<Proposal<'a>, GameError> {
        log::debug!(
            "Getting a proposal from {} for mission {}, proposal {}",
            self.game.name(from),
            mission,
            proposal
        );
        self.interactions
            .send(Message::NextProposal {
                proposer: from,
                mission,
                proposal,
            })
            .await?;

        let size = self.game.spec.mission_size(mission);
        let players = self
            .interactions
            .receive(|player, action| {
                if player != from {
                    Err("It's not your proposal!".to_string())
                } else {
                    match action {
                        Action::Propose { players } if players.len() == size => Ok(players),
                        Action::Propose { .. } => {
                            Err(format!("Proposal must contain {} players", size))
                        }
                        _ => Err("You can't do that right now".to_string()),
                    }
                }
            })
            .await?;

        self.interactions
            .send(Message::ProposalMade {
                proposer: from,
                players: players.clone(),
                mission,
                proposal,
            })
            .await?;

        let prop = Proposal {
            proposer: from,
            players,
            game: self.game,
        };

        log::debug!("Got {}", prop);

        Ok(prop)
    }

    /// Collect votes for a mission proposal. Players should assume they are voting on the most recent proposal,
    /// except for on mission 1 where players choose between two proposals. Each player must vote exactly once.
    async fn vote(&mut self) -> Result<VotingResults, GameError> {
        log::debug!("Voting on a proposal!");
        let game = self.game;
        self.interactions.send(Message::CommenceVoting).await?;
        let mut votes = HashMap::with_capacity(self.game.size());
        while votes.len() != self.game.size() {
            self.interactions
                .receive(|player, action| match action {
                    Action::Vote { upvote } => match votes.entry(player) {
                        Entry::Occupied(_) => Err("You already voted".to_string()),
                        Entry::Vacant(entry) => {
                            entry.insert(upvote);
                            log::debug!(
                                "{} {}",
                                game.name(player),
                                if upvote { "upvoted" } else { "downvoted" }
                            );
                            Ok(())
                        }
                    },
                    _ => Err("You can't do that right now".to_string()),
                })
                .await?;
        }

        let results = VotingResults::new(votes);
        self.interactions
            .send(Message::VotingResults {
                upvotes: results.upvotes().collect(),
                downvotes: results.downvotes().collect(),
                sent: results.sent,
            })
            .await?;

        Ok(results)
    }

    ///Run a mission, given the proposed players. This will return `true` if the mission passes, `false` if it fails.
    async fn send_mission(
        &mut self,
        proposal: Proposal<'_>,
        mission: usize,
    ) -> Result<bool, GameError> {
        // TODO: there should probably be a "Going on a Mission" message
        let game = self.game; // So we can use it inside the receive callback while mutably borrowing interactions
        log::debug!("Sending {} on mission {}", proposal, mission);

        let mut cards = HashMap::with_capacity(proposal.players.len());
        while cards.len() != proposal.players.len() {
            self.interactions
                .receive(|player, action| {
                    if proposal.players.contains(&player) {
                        match action {
                            Action::Play { card } => match cards.entry(player) {
                                Entry::Occupied(entry) => {
                                    Err(format!("You already played a {}", entry.get()))
                                }
                                Entry::Vacant(entry) => {
                                    let role = game.players[player].role;
                                    if role.can_play(card) {
                                        entry.insert(card);
                                        log::debug!("{} played a {}", game.name(player), card);
                                        Ok(())
                                    } else {
                                        Err(format!("You can't play a {}", card))
                                    }
                                }
                            },
                            // TODO: fun on-mission actions
                            _ => Err("You can't do that right now".to_string()),
                        }
                    } else {
                        Err("You are not on the mission".to_string())
                    }
                })
                .await?;
        }

        let failed = is_failure(game.spec, mission, cards.values());
        log::debug!(
            "Mission {} {}",
            mission,
            if failed { "failed" } else { "passed" }
        );

        self.interactions
            .send(Message::MissionResults {
                passed: !failed,
                successes: cards.values().filter(|c| c == &&Card::Success).count(),
                fails: cards.values().filter(|c| c == &&Card::Fail).count(),
                reverses: cards.values().filter(|c| c == &&Card::Reverse).count(),
            })
            .await?;

        if failed {
            let agravaine_declared = match time::timeout(
                DECLARE_DELAY,
                self.interactions.receive(|player, action| {
                    let role = game.players[player].role;
                    match action {
                        Action::Declare if role == Role::Agravaine => Ok(true),
                        _ => Err("You can't do that right now".to_string()),
                    }
                }),
            )
            .await
            {
                Ok(Ok(declared)) => declared,
                Ok(Err(err)) => return Err(err),
                // Timed out waiting for a declaration
                Err(_) => false,
            };

            if agravaine_declared {
                log::debug!("Agravaine declared!");

                // maybe this should be a different message?
                self.interactions
                    .send(Message::MissionResults {
                        passed: false,
                        successes: cards.values().filter(|c| c == &&Card::Success).count(),
                        fails: cards.values().filter(|c| c == &&Card::Fail).count(),
                        reverses: cards.values().filter(|c| c == &&Card::Reverse).count(),
                    })
                    .await?;

                return Ok(false);
            }
        }

        Ok(!failed)
    }
}

// Maybe a typemap for per-role game state?
// can have a base trait for callbacks (ex. reset Maeve's counter every round)

/// Tests if a mission has failed
fn is_failure<'a, I: IntoIterator<Item = &'a Card>>(
    spec: &GameSpec,
    mission: usize,
    cards: I,
) -> bool {
    let (fails, reverses) = cards
        .into_iter()
        .fold((0, 0), |(fails, reverses), card| match card {
            Card::Fail => (fails + 1, reverses),
            Card::Reverse => (fails, reverses + 1),
            Card::Success => (fails, reverses),
        });

    // Two reverses cancel each other out
    let reversed = reverses % 2 == 1;

    if mission == 4 && spec.double_fail_mission_four {
        // Mission 4 can fail if there are 2+ fails or a reverse and a fail, in large enough games
        (fails >= 2 && !reversed) || (fails == 1 && reverses == 1)
    } else {
        // Normally, missions fail if they contain a fail OR are reversed
        (fails > 0) ^ reversed
    }
}

impl VotingResults {
    fn new(votes: HashMap<PlayerId, bool>) -> VotingResults {
        let mut net = 0; // Will be above 0 if there are more upvotes than downvotes
        for vote in votes.values() {
            if *vote {
                net += 1;
            } else {
                net -= 1;
            }
        }

        VotingResults {
            votes,
            sent: net > 0,
        }
    }

    /// Players who upvoted the proposal
    fn upvotes<'a>(&'a self) -> impl Iterator<Item = PlayerId> + 'a {
        self.votes
            .iter()
            .filter_map(|(player, vote)| if *vote { Some(*player) } else { None })
    }

    /// Players who downvoted the proposal
    fn downvotes<'a>(&'a self) -> impl Iterator<Item = PlayerId> + 'a {
        self.votes
            .iter()
            .filter_map(|(player, vote)| if !*vote { Some(*player) } else { None })
    }
}

impl<'a> fmt::Display for Proposal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.players
            .iter()
            .format_with(", ", |id, f| f(&self.game.name(*id)))
            .fmt(f)?;
        write!(f, " (proposed by {})", self.game.name(self.proposer))
    }
}

#[test]
fn test_is_failure() {
    use super::GameSpec;
    let spec = GameSpec::for_players(5);

    assert!(!is_failure(
        spec,
        1,
        &[Card::Success, Card::Success, Card::Success]
    ));
    assert!(!is_failure(
        spec,
        1,
        &[Card::Success, Card::Fail, Card::Reverse]
    ));
    assert!(!is_failure(
        spec,
        1,
        &[Card::Success, Card::Reverse, Card::Reverse]
    ));

    assert!(is_failure(
        spec,
        1,
        &[Card::Success, Card::Success, Card::Fail]
    ));
    assert!(is_failure(
        spec,
        1,
        &[Card::Success, Card::Fail, Card::Fail]
    ));
    assert!(is_failure(spec, 1, &[Card::Fail, Card::Fail, Card::Fail]));

    assert!(is_failure(
        spec,
        1,
        &[Card::Success, Card::Success, Card::Reverse]
    ));
    assert!(is_failure(
        spec,
        1,
        &[Card::Fail, Card::Reverse, Card::Reverse]
    ));

    // TODO: replace with proper 7-player spec
    let mut with_seven = spec.clone();
    with_seven.double_fail_mission_four = true;

    assert!(!is_failure(
        &with_seven,
        4,
        &[Card::Success, Card::Success, Card::Success, Card::Fail]
    ));
    assert!(!is_failure(
        &with_seven,
        4,
        &[Card::Success, Card::Reverse, Card::Fail, Card::Fail]
    ));
    assert!(!is_failure(
        &with_seven,
        4,
        &[Card::Success, Card::Success, Card::Success, Card::Reverse]
    ));
    assert!(!is_failure(
        &with_seven,
        4,
        &[Card::Reverse, Card::Reverse, Card::Fail, Card::Success]
    ));

    assert!(is_failure(
        &with_seven,
        4,
        &[Card::Success, Card::Success, Card::Fail, Card::Fail]
    ));
    // This is why reversing mission 4 as Lance is a choice
    assert!(is_failure(
        &with_seven,
        4,
        &[Card::Success, Card::Success, Card::Reverse, Card::Fail]
    ));
    assert!(is_failure(
        &with_seven,
        4,
        &[Card::Reverse, Card::Reverse, Card::Fail, Card::Fail]
    ));
}

#[cfg(test)]
mod test {
    use futures::executor::block_on;
    use maplit::{hashmap, hashset};

    use super::super::{Player, Players, Role};
    use super::super::interactions::test::TestInteractions;
    use super::super::messages::{Action, Message};
    use super::*;

    fn make_game() -> Game {
        let mut players = Players::new();
        players.add_player(Player {
            id: 1,
            role: Role::Merlin,
            name: "Player 1".to_string(),
        });
        players.add_player(Player {
            id: 2,
            role: Role::Lancelot,
            name: "Player 2".to_string(),
        });
        players.add_player(Player {
            id: 3,
            role: Role::Iseult,
            name: "Player 3".to_string(),
        });
        players.add_player(Player {
            id: 4,
            role: Role::Maelegant,
            name: "Player 4".to_string(),
        });
        players.add_player(Player {
            id: 5,
            role: Role::Mordred,
            name: "Player 5".to_string(),
        });

        Game {
            players,
            info: hashmap! {
                1 => "You are Merlin".to_string(),
                2 => "You are Lancelot".to_string(),
                3 => "You are Iseult".to_string(),
                4 => "You are Maelegant".to_string(),
                5 => "You are Mordred".to_string(),
            },
            proposal_order: vec![1, 2, 3, 4, 5],
            spec: GameSpec::for_players(5),
        }
    }

    #[test]
    fn test_proposals() {
        let mut interactions = TestInteractions::new();
        let game = make_game();
        let mut engine = GameEngine::new(&game, &mut interactions);

        engine.interactions.extend_actions(vec![
            (
                1,
                Action::Propose {
                    players: hashset![1, 3],
                },
            ),
            (
                2,
                Action::Propose {
                    players: hashset![2, 4, 1],
                },
            ),
            (
                2,
                Action::Propose {
                    players: hashset![2, 4],
                },
            ),
        ]);

        let proposal = block_on(engine.get_proposal(2, 1, 1)).unwrap();

        assert_eq!(proposal.proposer, 2);
        assert_eq!(proposal.players, hashset![2, 4]);

        assert_eq!(
            interactions.messages_for(1).collect::<Vec<_>>(),
            vec![&Message::Error("It's not your proposal!".to_string())]
        );

        assert_eq!(
            interactions.messages_for(2).collect::<Vec<_>>(),
            vec![&Message::Error(
                "Proposal must contain 2 players".to_string()
            )]
        );

        assert_eq!(
            interactions.broadcasts(),
            &[
                Message::NextProposal {
                    proposer: 2,
                    mission: 1,
                    proposal: 1
                },
                Message::ProposalMade {
                    proposer: 2,
                    mission: 1,
                    proposal: 1,
                    players: hashset![2, 4]
                }
            ]
        );
    }

    #[test]
    fn test_voting_sent() {
        let mut interactions = TestInteractions::new();
        let game = make_game();
        let mut engine = GameEngine::new(&game, &mut interactions);

        engine.interactions.extend_actions(vec![
            (2, Action::Vote { upvote: true }),
            (3, Action::Vote { upvote: false }),
            (1, Action::Vote { upvote: true }),
            (2, Action::Vote { upvote: false }),
            (4, Action::Vote { upvote: false }),
            (5, Action::Vote { upvote: true }),
        ]);

        let results = block_on(engine.vote()).unwrap();
        assert!(results.sent);
        assert_eq!(results.upvotes().collect::<HashSet<_>>(), hashset![2, 1, 5]);
        assert_eq!(results.downvotes().collect::<HashSet<_>>(), hashset![3, 4]);

        assert_eq!(
            interactions.messages_for(2).collect::<Vec<_>>(),
            vec![&Message::Error("You already voted".to_string())]
        );

        assert_eq!(interactions.broadcasts()[0], Message::CommenceVoting);
        if let Message::VotingResults {
            upvotes,
            downvotes,
            sent,
        } = interactions.broadcasts()[1].clone()
        {
            assert!(sent);
            assert_eq!(
                upvotes.into_iter().collect::<HashSet<_>>(),
                hashset![2, 1, 5]
            );
            assert_eq!(
                downvotes.into_iter().collect::<HashSet<_>>(),
                hashset![3, 4]
            );
        } else {
            panic!("Expected a VotingResults message");
        }
    }

    // TODO: ties
}
