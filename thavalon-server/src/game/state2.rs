#![allow(dead_code)]
use std::collections::{HashMap, HashSet};
use std::fmt;

use super::messages::{Action, Message, VoteCounts};
use super::role::Role;
use super::{Card, Game, GameSpec, MissionNumber};

// Use Hoverbear pattern with an enum wrapper that rejects invalid (state, action) combinations and an Effect enum

/// Result of handling a player action. The [`GameStateWrapper`] is the new state of the game and the [`Effect`]
/// [`Vec`] describes side-effects of the state transition.
type ActionResult = (GameStateWrapper, Vec<Effect>);

/// Wrapper over specific phases, so callers can hold a game in any phase.
enum GameStateWrapper {
    Proposing(GameState<Proposing>),
    Voting(GameState<Voting>),
    OnMission(GameState<OnMission>),
}

/// State of an in-progress game. Game state is divided into two parts. Data needed in all phases of the game,
/// such as player information, is stored in the `GameState` directly. Phase-specific data, such as how players
/// voted on a specific mission proposal, is stored in a particular [`Phase`] implementation.
struct GameState<P: Phase> {
    /// State specific to the current game phase.
    phase: P,
    /// Game configuration
    game: Game,
    /// All proposals made in the game
    proposals: Vec<Proposal>,
    /// Results of all completed missions
    mission_results: Vec<MissionResults>,
}

/// A phase of the THavalon state machine
trait Phase: Sized {
    /// Lifts a game in this phase back into the [`GameStateWrapper`] enum. This is used by code
    /// that is generic over different game phases and needs a wrapped version of the game.
    fn wrap(game: GameState<Self>) -> GameStateWrapper;
}

/// A side-effect of a state transition. In most cases, this will result in sending a message to some or all players.
enum Effect {
    Reply(Message),
    Broadcast(Message),
    StartTimeout,
    ClearTimeout,
}

/// Phase for waiting for a player to make a mission proposal.
struct Proposing {
    /// The player whose turn it is to propose
    proposer: String,
}

/// Phase for voting on a mission proposal
struct Voting {
    /// Map from players to whether they up- or down-voted the proposal.
    votes: HashMap<String, bool>,
}

/// Phase for when the voted-on players are going on a mission
struct OnMission {
    /// The proposal this mission is based on. Used to check if players are on the mission or not
    proposal_index: usize,

    /// Cards played by each player on the mission.
    cards: HashMap<String, Card>,

    /// The number of questing beasts played
    questing_beasts: usize,
}

struct Proposal {
    proposer: String,
    players: HashSet<String>,
}

struct MissionResults {
    passed: bool,
    players: HashSet<String>,
}

impl GameState<Proposing> {
    /// Respond to a proposal. If the proposal is invalid, or it's not the player's turn to propose, this returns
    /// an error reply. Otherwise, the proposal is sent to all players and the game moves into the [`Voting`] phase.
    /// On mission 1, we instead ask for two proposals before voting. Once force is active, we skip the voting phase
    /// and go straight to [`OnMission`].
    fn handle_proposal(self, player: &str, proposal: HashSet<String>) -> ActionResult {
        if player != self.phase.proposer {
            return self.player_error("It's not your proposal");
        }

        let mission = self.mission();
        let expected_size = self.game.spec.mission_size(mission);
        if proposal.len() != expected_size {
            return self.player_error(format!("Proposal must contain {} players", expected_size));
        }

        for player in proposal.iter() {
            if self.game.players.by_name(player).is_none() {
                return self.player_error(format!("{} is not in the game", player));
            }
        }

        use itertools::Itertools;
        log::debug!(
            "{} proposed {} for mission {}",
            player,
            proposal.iter().format(", "),
            mission
        );

        let mut effects = vec![Effect::Broadcast(Message::ProposalMade {
            proposer: player.to_string(),
            mission,
            players: proposal.clone(),
        })];

        let spent_proposals = self.spent_proposals();

        let GameState {
            game,
            mut proposals,
            mission_results,
            ..
        } = self;
        proposals.push(Proposal {
            proposer: player.to_string(),
            players: proposal,
        });

        if mission == 1 && proposals.len() == 1 {
            // On mission 1, we need the first 2 proposals before voting
            log::debug!("Getting a second proposal for mission 1");
            let next_proposer = game.next_proposer(player).to_string();
            effects.push(Effect::Broadcast(Message::NextProposal {
                proposer: next_proposer.clone(),
                mission,
                proposals_made: spent_proposals,
                max_proposals: game.spec.max_proposals,
            }));

            let next_game = GameStateWrapper::Proposing(GameState {
                game,
                proposals,
                mission_results,
                phase: Proposing::new(next_proposer),
            });
            (next_game, effects)
        } else if spent_proposals > game.spec.max_proposals {
            let proposal = proposals.last().unwrap();
            log::debug!("Sending {} with force for mission {}", proposal, mission);
            effects.push(Effect::Broadcast(Message::MissionGoing {
                mission,
                players: proposal.players.clone(),
            }));
            let next_game = GameStateWrapper::OnMission(GameState {
                phase: OnMission::new(proposals.len() - 1),
                game,
                proposals,
                mission_results,
            });
            (next_game, effects)
        } else {
            if mission == 1 {
                let first_proposal = &proposals[proposals.len() - 2];
                let second_proposal = proposals.last().unwrap();
                log::debug!(
                    "Voting on {} and {} for mission {}",
                    first_proposal,
                    second_proposal,
                    mission
                );
            } else {
                let proposal = proposals.last().unwrap();
                log::debug!("Voting on {} for mission {}", proposal, mission);
            }

            effects.push(Effect::Broadcast(Message::CommenceVoting));
            let next_game = GameStateWrapper::Voting(GameState {
                game,
                proposals,
                mission_results,
                phase: Voting::new(),
            });
            (next_game, effects)
        }
    }
}

impl Proposing {
    /// Create a new `Proposing` phase given the player proposing.
    pub fn new(proposer: String) -> Proposing {
        Proposing { proposer }
    }
}

impl GameState<Voting> {
    fn handle_vote(mut self, player: &str, is_upvote: bool) -> ActionResult {
        if self.phase.votes.contains_key(player) {
            return self.player_error("You already voted");
        }

        log::debug!(
            "{} {}",
            player,
            if is_upvote { "upvoted" } else { "downvoted" }
        );
        self.phase.votes.insert(player.to_string(), is_upvote);

        if self.phase.votes.len() == self.game.size() {
            let mission = self.mission();
            let spent_proposals = self.spent_proposals();
            let GameState {
                game,
                proposals,
                mission_results,
                phase: Voting { votes },
            } = self;

            let mut upvotes = HashSet::new();
            let mut downvotes = HashSet::new();

            for (player, vote) in votes.into_iter() {
                if vote {
                    upvotes.insert(player);
                } else {
                    downvotes.insert(player);
                }
            }

            let sent = upvotes.len() > downvotes.len();
            let mut effects = vec![Effect::Broadcast(Message::VotingResults {
                sent,
                counts: VoteCounts::Public { upvotes, downvotes },
            })];

            if mission == 1 {
                let proposal_index = if sent { 0 } else { 1 };
                let proposal = &proposals[proposal_index];
                log::debug!("Voted to send {} on mission 1", proposal);
                effects.push(Effect::Broadcast(Message::MissionGoing {
                    mission,
                    players: proposal.players.clone(),
                }));
                let next_game = GameStateWrapper::OnMission(GameState {
                    phase: OnMission::new(proposal_index),
                    game,
                    proposals,
                    mission_results,
                });
                (next_game, effects)
            } else {
                let proposal = proposals.last().expect("Voted with no proposals!");
                if sent {
                    log::debug!("Voted to send {} on mission {}", proposal, mission);
                    effects.push(Effect::Broadcast(Message::MissionGoing {
                        mission,
                        players: proposal.players.clone(),
                    }));
                    let next_game = GameStateWrapper::OnMission(GameState {
                        phase: OnMission::new(proposals.len() - 1),
                        game,
                        proposals,
                        mission_results,
                    });
                    (next_game, effects)
                } else {
                    log::debug!("Voted not to send {}", proposal);
                    let next_proposer = game.next_proposer(&proposal.proposer);
                    effects.push(Effect::Broadcast(Message::NextProposal {
                        proposer: next_proposer.to_string(),
                        mission,
                        proposals_made: spent_proposals,
                        max_proposals: game.spec.max_proposals,
                    }));
                    let next_game = GameStateWrapper::Proposing(GameState {
                        phase: Proposing::new(next_proposer.to_string()),
                        game,
                        proposals,
                        mission_results,
                    });
                    (next_game, effects)
                }
            }
        } else {
            // If we don't have all the votes yet, there's no state change
            (GameStateWrapper::Voting(self), vec![])
        }
    }
}

impl Voting {
    /// Create a new `Voting` phase with no votes yet cast.
    fn new() -> Voting {
        Voting {
            votes: HashMap::new(),
        }
    }
}

impl GameState<OnMission> {
    fn handle_card(mut self, player: &str, card: Card) -> ActionResult {
        if self.includes_player(player) {
            if let Some(card) = self.phase.cards.get(player).cloned() {
                self.player_error(format!("You already played a {}", card))
            } else {
                self.phase.cards.insert(player.to_string(), card);
                log::debug!("{} played a {}", player, card);

                if self.phase.cards.len() == self.proposal().players.len() {
                    let mission = self.mission();
                    let passed =
                        !is_failure(self.game.spec, mission as usize, self.phase.cards.values());
                    log::debug!(
                        "Mission {} {}",
                        mission,
                        if passed { "passed" } else { "failed" }
                    );

                    self.mission_results.push(MissionResults {
                        passed,
                        players: self.proposal().players.clone(),
                    });

                    let (mut successes, mut fails, mut reverses) = (0, 0, 0);
                    for card in self.phase.cards.values() {
                        match card {
                            Card::Success => successes += 1,
                            Card::Fail => fails += 1,
                            Card::Reverse => reverses += 1,
                        }
                    }

                    let mut effects = vec![Effect::Broadcast(Message::MissionResults {
                        mission,
                        successes,
                        fails,
                        reverses,
                        questing_beasts: self.phase.questing_beasts,
                        passed,
                    })];

                    if self.game.spec.has_role(Role::Agravaine) {
                        todo!()
                    } else {
                        let next_proposer = self
                            .game
                            .next_proposer(&self.proposal().proposer)
                            .to_string();
                        effects.push(Effect::Broadcast(Message::NextProposal {
                            proposer: next_proposer.clone(),
                            mission: self.mission(),
                            // This will be accurate because we've already added in the mission results
                            proposals_made: self.spent_proposals(),
                            max_proposals: self.game.spec.max_proposals,
                        }));

                        let next_state = GameState {
                            proposals: self.proposals,
                            mission_results: self.mission_results,
                            game: self.game,
                            phase: Proposing::new(next_proposer),
                        };
                        (GameStateWrapper::Proposing(next_state), effects)
                    }
                } else {
                    // If cards aren't all in yet, there's no state change
                    (GameStateWrapper::OnMission(self), vec![])
                }
            }
        } else {
            self.player_error("You're not on the mission")
        }
    }

    fn handle_questing_beast(mut self, player: &str) -> ActionResult {
        if self.includes_player(player) {
            log::debug!("{} played a questing beast", player);
            self.phase.questing_beasts += 1;
            (GameStateWrapper::OnMission(self), vec![])
        } else {
            self.player_error("You're not on the mission")
        }
    }

    /// The propopsal this mission is based on
    fn proposal(&self) -> &Proposal {
        self.proposals
            .get(self.phase.proposal_index)
            .expect("On a mission with an invalid proposal index!")
    }

    fn includes_player(&self, player: &str) -> bool {
        self.proposal().players.contains(player)
    }
}

impl OnMission {
    /// Create a new `OnMission` phase given the proposal used for it
    fn new(proposal_index: usize) -> OnMission {
        OnMission {
            proposal_index,
            cards: HashMap::new(),
            questing_beasts: 0,
        }
    }
}

/// Tests if a mission has failed
/// Note: this is written the way it is because it's easier to express the rules for a mission failing. In general,
/// it's clearer to keep track of whether or not it passed.
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

// Boilerplate for wrapping game phases into an enum

macro_rules! impl_phase {
    ($phase:ident) => {
        impl Phase for $phase {
            fn wrap(game: GameState<Self>) -> GameStateWrapper {
                GameStateWrapper::$phase(game)
            }
        }
    };
}

impl_phase!(Proposing);
impl_phase!(Voting);
impl_phase!(OnMission);

// Convenience methods shared across game phases
impl<P: Phase> GameState<P> {
    /// Generate an [`ActionResult`] that keeps the current state and returns an error reply to the player.
    fn player_error<S: Into<String>>(self, message: S) -> ActionResult {
        (P::wrap(self), vec![player_error(message)])
    }

    /// The current mission, indexed starting at 1
    fn mission(&self) -> MissionNumber {
        self.mission_results.len() as u8 + 1
    }

    /// Calculates the number of "spent" proposals, for the purposes of determining if force is active
    /// - The two proposals on mission 1 do not count
    /// - Proposals that are sent do not count. Equivalently, every time a mission is sent we get a proposal back
    ///
    /// *This will be off by 1 while going on a mission, since the spent proposal has not yet been returned*
    fn spent_proposals(&self) -> usize {
        self.proposals
            .len()
            .saturating_sub(2) // Subtract 2 proposals for mission 1
            .saturating_sub(self.mission_results.len()) // Subtract 1 proposal for each sent mission
    }
}

impl GameStateWrapper {
    /// Advance to the next game state given a player action
    fn handle_action(self, player: &str, action: Action) -> ActionResult {
        log::debug!("Responding to {:?} from {}", action, player);
        match (self, action) {
            (GameStateWrapper::Proposing(inner), Action::Propose { players }) => {
                inner.handle_proposal(player, players)
            }
            (GameStateWrapper::Voting(inner), Action::Vote { upvote }) => {
                inner.handle_vote(player, upvote)
            }
            (GameStateWrapper::OnMission(inner), Action::QuestingBeast) => {
                inner.handle_questing_beast(player)
            }
            (state, _) => (state, vec![player_error("You can't do that right now")]),
        }
    }
}

impl fmt::Display for Proposal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use itertools::Itertools;
        write!(
            f,
            "{} (proposed by {})",
            self.players.iter().format(", "),
            self.proposer
        )
    }
}

/// Generate an [`Effect`] that sends an error reply to the player.
fn player_error<S: Into<String>>(message: S) -> Effect {
    Effect::Reply(Message::Error(message.into()))
}

mod test {
    use super::super::{Card, GameSpec};
    use super::is_failure;

    #[test]
    fn test_is_failure() {
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
}
