use std::collections::HashMap;
use std::time::Duration;

use super::prelude::*;
use super::RoleState;

/// Phase for when the voted-on players are going on a mission
pub struct OnMission {
    /// The proposal this mission is based on. Used to check if players are on the mission or not
    proposal_index: usize,

    /// Cards played by each player on the mission.
    cards: HashMap<String, Card>,

    /// The number of questing beasts played
    questing_beasts: usize,
}

/// Placeholder phase used when waiting for Agravaine to declare
pub struct WaitingForAgravaine {
    proposal_index: usize,
}

/// How long to wait for an Agravaine declaration
const AGRAVAINE_TIMEOUT: Duration = Duration::from_secs(30);

impl GameState<OnMission> {
    pub fn handle_card(mut self, player: &str, card: Card) -> ActionResult {
        if self.includes_player(player) {
            if let Some(card) = self.phase.cards.get(player).cloned() {
                self.player_error(format!("You already played a {}", card))
            } else if !self
                .game
                .players
                .by_name(player)
                .unwrap()
                .role
                .can_play(card)
            {
                self.player_error(format!("You can't play a {}", card))
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
                    self.add_lover_effects(&mut effects);

                    // TODO: how does Agravaine work on mission 4?
                    if self.game.spec.has_role(Role::Agravaine) && passed && fails != 0 {
                        effects.push(Effect::StartTimeout(AGRAVAINE_TIMEOUT));
                        let next_phase = WaitingForAgravaine {
                            proposal_index: self.phase.proposal_index,
                        };
                        (
                            GameStateWrapper::WaitingForAgravaine(self.with_phase(next_phase)),
                            effects,
                        )
                    } else {
                        let proposal_index = self.phase.proposal_index;
                        conclude_mission(self, effects, proposal_index)
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

    pub fn handle_questing_beast(mut self, player: &str) -> ActionResult {
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

    // Append messages for lovers to an exisiting list of effects. This will not modify effects if there are no lovers in game,
    // so this is safe to call in any game.
    fn add_lover_effects(&self, effects: &mut Vec<Effect>) {
        let tristan_display_name = self.game.display_name_from_role(Role::Tristan);
        let tristan_on_mission =
            tristan_display_name.map_or(None, |x| Some(self.proposal().players.contains(x)));
        let iseult_display_name = self.game.display_name_from_role(Role::Iseult);
        let iseult_on_mission =
            iseult_display_name.map_or(None, |x| Some(self.proposal().players.contains(x)));
        // First, add messages going to Tristan. If there is no Tristan, do nothing.
        if let Some(tristan_display_name) = tristan_display_name {
            effects.push(self.build_lover_effect(
                tristan_display_name,
                tristan_on_mission.unwrap(),
                iseult_display_name,
                iseult_on_mission,
            ));
        }
        // Next do the same for Iseult.
        if let Some(iseult_display_name) = iseult_display_name {
            effects.push(self.build_lover_effect(
                iseult_display_name,
                iseult_on_mission.unwrap(),
                tristan_display_name,
                tristan_on_mission,
            ));
        }
    }

    // Builds a message for a player, given information about their lover who may not exist.
    fn build_lover_effect(
        &self,
        player_display_name: &str,
        player_on_mission: bool,
        opt_lover_display_name: Option<&String>,
        opt_lover_on_mission: Option<bool>,
    ) -> Effect {
        let inner_message =
            opt_lover_on_mission.map_or("does not exist. You're alone :'(".to_owned(), |x| {
                if x {
                    if player_on_mission {
                        format!(
                            "was on the mission with you, it's {}",
                            opt_lover_display_name.unwrap()
                        )
                    } else {
                        "was on this mission.".to_owned()
                    }
                } else {
                    "was not on this mission.".to_owned()
                }
            });
        let toast_message = ["Your lover ", &inner_message].concat();
        Effect::Send(
            player_display_name.to_string(),
            Message::Toast {
                severity: ToastSeverity::INFO,
                message: toast_message,
            },
        )
    }
}

impl OnMission {
    /// Create a new `OnMission` phase given the proposal used for it
    pub fn new(proposal_index: usize) -> OnMission {
        OnMission {
            proposal_index,
            cards: HashMap::new(),
            questing_beasts: 0,
        }
    }
}

impl GameState<WaitingForAgravaine> {
    pub fn handle_declaration(mut self, player: &str) -> ActionResult {
        // Subtract one since the mission Agravaine failing is the one prior to the current propposal.
        let mission_number = self.mission() - 1;
        let mission = self
            .mission_results
            .last_mut()
            .expect("Waiting for Agravaine but no mission went");
        if mission.players.contains(player) && self.game.players.is(player, Role::Agravaine) {
            log::debug!(
                "Agravaine declaration by {} caused mission {} to fail",
                player,
                mission_number
            );
            mission.passed = false;

            let effects = vec![
                Effect::Broadcast(Message::AgravaineDeclaration {
                    mission: mission_number,
                    player: player.to_string(),
                }),
                Effect::Broadcast(Message::Toast {
                    severity: ToastSeverity::URGENT,
                    message: format!("{} has declared as Agravaine!", player),
                }),
                Effect::ClearTimeout,
            ];

            let proposal = self.phase.proposal_index;
            conclude_mission(self, effects, proposal)
        } else {
            self.player_error("You can't declare right now")
        }
    }

    pub fn handle_timeout(self) -> ActionResult {
        log::debug!("Timed out waiting for Agravaine to declare");
        let proposal = self.phase.proposal_index;
        conclude_mission(self, vec![], proposal)
    }
}

impl_phase!(OnMission);
impl_phase!(WaitingForAgravaine);

/// Common logic for transitioning to the next phase after a mission ends. This is shared by the [`OnMission`] and
/// [`WaitingForAgravaine`] phases. This assumes that `state.mission_results` is up-to-date (including Agravaine
/// declarations).
///
/// # Arguments
/// * `state` - the `GameState` to transition from (either `OnMission` or `WaitingForAgravaine`)
/// * `effects` - additional side-effects to apply (this varies depending on whether or not Agravaine declared)
/// * `proposal` - the proposal the mission was based on, used to figure out who is proposing next
fn conclude_mission<P: Phase>(
    mut state: GameState<P>,
    mut effects: Vec<Effect>,
    proposal: usize,
) -> ActionResult {
    let (mut successes, mut fails) = (0, 0);
    for mission in state.mission_results.iter() {
        if mission.passed {
            successes += 1
        } else {
            fails += 1
        }
    }

    if successes == 3 {
        log::debug!("3 missions have passed, moving to assassination");
        effects.push(Effect::Broadcast(Message::BeginAssassination {
            assassin: state.game.assassin.to_string(),
        }));
        let next_state = GameStateWrapper::Assassination(state.with_phase(Assassination {}));
        (next_state, effects)
    } else if fails == 3 {
        log::debug!("3 missions have failed, the Evil team has won");
        state.into_done(Team::Evil, effects)
    } else {
        let mission = state.mission();
        let next_proposer = if mission == 2 {
            // On mission 2, the third proposer goes first, because the first 2 proposers already proposed for mission 1
            // the mod is to account for 2-player testing games, for which we wrap back around to player 1
            // Note: the index of the 3rd proposer is 2
            let next_proposer_index = 2 % state.game.proposal_order.len();
            state.game.proposal_order[next_proposer_index].clone()
        } else {
            let mission_proposer = &state.proposals[proposal].proposer;
            state.game.next_proposer(mission_proposer).to_string()
        };

        RoleState::on_round_start(&mut state, &mut effects);
        state.into_proposing(next_proposer, effects)
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

#[cfg(test)]
mod test {
    use super::super::prelude::*;
    use super::is_failure;

    #[test]
    fn test_is_failure() {
        let spec = GameSpec::for_players(5).unwrap();

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
