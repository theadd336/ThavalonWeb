import random
from .player import Player
from game.roles.iseult import Iseult
from game.roles.lancelot import Lancelot
from game.roles.maelegant import Maelegant
from game.roles.merlin import Merlin
from game.roles.mordred import Mordred
from game.roles.morgana import Morgana
from game.roles.percival import Percival
from game.roles.tristan import Tristan
from game.game_constants import GamePhase, LobbyStatus, MissionResult, MissionCard
from typing import Any, Dict, List

_MIN_NUM_PLAYERS = 2
_MAX_NUM_PLAYERS = 10

_MISSION_NUM_TO_PROPOSAL_SIZE = {
    1: [1, 1, 1, 1, 1],
    2: [1, 1, 2, 2, 2],
    3: [1, 2, 3, 3, 3],
    4: [1, 2, 3, 2, 3],
    5: [2, 3, 2, 3, 3],
    6: [2, 3, 4, 3, 4],
    7: [2, 3, 3, 4, 4],
    8: [3, 4, 4, 5, 5],
    9: [3, 4, 4, 5, 5],
    10: [3, 4, 4, 5, 5]
}

_GAME_SIZE_TO_GOOD_COUNT = {
    1: 0,
    2: 2,
    3: 2,
    4: 2,
    5: 3,
    6: 4,
    7: 4,
    8: 5,
    9: 6,
    10: 6
}

_GOOD_ROLES = [Iseult, Lancelot, Merlin, Percival, Tristan]
_EVIL_ROLES = [Maelegant, Mordred, Morgana]


class Game:
    def __init__(self) -> None:
        # Main Game Info
        # map session id to actual player object
        self.session_id_to_player: Dict[str, Player] = {}
        # map player name to session_id
        self.player_name_to_session_id: Dict[str, str] = {}
        # current state of game
        self.lobby_status: LobbyStatus = LobbyStatus.JOINING
        # current phase when game is in progress
        self.game_phase: GamePhase = GamePhase.PROPOSAL
        # mission number results
        self.mission_num_to_result: Dict[int, MissionResult] = {}
        # mission number to players on mission
        self.mission_players: Dict[int, List[str]] = {}
        # for declarations, index in proposer (0-indexed) to declared role name
        self.declarations: Dict[int, str] = {}
        # last vote info in game, mapping player to vote
        self.last_vote_info: Dict[str, bool] = {}

        # Information about proposals
        # the proposal, or seating, order of players, one for names and one for players
        self.proposal_order_names: List[str] = []
        self.proposal_order_players: List[Player] = []
        # the index number of the current proposer(s), 0-indexed
        self.proposer_index = 0
        # the id of the current proposer
        self.proposer_id: str = ""
        # the current proposal number, 1-indexed.
        self.current_proposal_num: int = 1
        # number of proposals in round 2-5. Round 1 is always 2 proposers
        self.max_num_proposers: int = 0
        # the current proposals. A list because in round 1, there can be 2 simultaneous proposals
        self.current_proposals: List[List[str]] = []

        # Information about voting on proposals
        # The number of votes made so far
        self.number_votes: int = 0

        # the current mission number, 0-indexed.
        self.mission_num: int = 0
        # players on current mission
        self.current_mission: List[str] = 0
        # count of cards played so far
        self.current_mission_count: int = 0

    # methods for adding players
    def get_num_players(self) -> int:
        return len(self.session_id_to_player)

    def is_game_full(self) -> bool:
        return self.get_num_players() == _MAX_NUM_PLAYERS

    def get_player_names_in_game(self) -> List[str]:
        return [player.name for player in self.session_id_to_player.values()]

    def add_player(self, session_id: str, name: str) -> List[str]:
        if self.lobby_status != LobbyStatus.JOINING:
            raise ValueError("Can only add player while in lobby.")
        if self.is_game_full():
            raise ValueError(f"Game currently has max {_MAX_NUM_PLAYERS} players, cannot add new player.")
        if session_id in self.session_id_to_player:
            raise ValueError(f"Session id {session_id} already in playermanager.")
        if name in [player.name for player in self.session_id_to_player.values()]:
            raise ValueError(f"Player with name {name} already in game.")
        if name in self.player_name_to_session_id:
            raise ValueError(f"Player with name {name} already in game.")
        player = Player(session_id, name)
        self.session_id_to_player[session_id] = player
        self.player_name_to_session_id[name] = session_id
        return [player.name for player in self.session_id_to_player.values()]

    def get_player(self, session_id: str) -> Player:
        if session_id not in self.session_id_to_player:
            raise ValueError(f"Player with session id {session_id} does not exist")
        return self.session_id_to_player[session_id]

    def is_player_in_game(self, session_id: str) -> bool:
        return session_id in self.session_id_to_player

    def remove_player(self, session_id: str) -> List[str]:
        if self.lobby_status != LobbyStatus.JOINING:
            raise ValueError("Can only remove player while in lobby.")
        if session_id not in self.session_id_to_player:
            raise ValueError(f"Player with session id {session_id} does not exist")
        player_name = self.session_id_to_player[session_id].name
        del self.player_name_to_session_id[player_name]
        del self.session_id_to_player[session_id]
        return [player.name for player in self.session_id_to_player.values()]

    # method for starting the game
    def start_game(self) -> None:
        # validate players
        num_players = self.get_num_players()
        if num_players < _MIN_NUM_PLAYERS:
            raise ValueError(f"Game must have at least {_MIN_NUM_PLAYERS} to be started")
        if num_players > _MAX_NUM_PLAYERS:
            raise ValueError(f"Game somehow has more than {_MAX_NUM_PLAYERS}, unable to start")

        # shuffle player in order
        players = list(self.session_id_to_player.values())
        random.shuffle(players)

        # proposal order is player names shuffled
        self.proposal_order_players = [player for player in players]
        random.shuffle(self.proposal_order_players)
        self.proposal_order_names = [player.name for player in self.proposal_order_players]

        # first two proposers are last 2 in proposal order
        self.proposer_index = num_players - 2  # next to last player proposes first, subtract to because 0-indexed
        self.proposer_id = self.proposal_order_players[-2].session_id

        # get number good/evil in game
        num_good = _GAME_SIZE_TO_GOOD_COUNT[num_players]
        num_evil = num_players - num_good

        # num proposers is number evil + 1 round for all rounds except round 1
        self.max_num_proposers = num_evil + 1

        # generate which good/evil roles are in game
        good_role_indices = random.sample(range(0, len(_GOOD_ROLES)), num_good)
        evil_role_indices = random.sample(range(0, len(_EVIL_ROLES)), num_evil)

        # get lover indices
        good_roles_in_game = [_GOOD_ROLES[idx] for idx in good_role_indices]
        iseult_idx = -1
        tristan_idx = -1
        try:
            iseult_idx = good_roles_in_game.index(Iseult)
        except ValueError:
            pass

        try:
            tristan_idx = good_roles_in_game.index(Tristan)
        except ValueError:
            pass

        # only care about cases where one lover is in the game
        if bool(iseult_idx == -1) != bool(tristan_idx == -1):
            lone_lover_idx = iseult_idx if iseult_idx != -1 else tristan_idx
            if random.choice([True, False]):
                # True - replace lone lover with new role
                lover_roles_not_in_game = list(set(_GOOD_ROLES) - set(good_roles_in_game) - {Iseult, Tristan})
                good_roles_in_game[lone_lover_idx] = random.choice(lover_roles_not_in_game)
            else:
                # False - replace another role with other lover
                other_lover = Iseult if iseult_idx == -1 else Tristan
                other_role_indices = list(range(num_good))
                other_role_indices.remove(lone_lover_idx)
                good_roles_in_game[random.choice(other_role_indices)] = other_lover

        # assign first N players a good role
        for player, good_role in zip(players[:num_good], good_roles_in_game):
            player.role = good_role()

        # assign rest of players an evil role
        for player, evil_role_index in zip(players[num_good:], evil_role_indices):
            player.role = _EVIL_ROLES[evil_role_index]()

        for index, player in enumerate(players):
            for other_player in players[index + 1:]:
                if player != other_player:
                    player.role.add_seen_player(other_player)
                    other_player.role.add_seen_player(player)

        self.lobby_status = LobbyStatus.IN_PROGRESS

    def get_player_info(self, session_id: str) -> Dict[str, Any]:
        if self.lobby_status != LobbyStatus.IN_PROGRESS:
            raise ValueError("Can only get player info when game in progress")
        player = self.get_player(session_id)
        return {
            "role": player.role.role_name,
            "team": player.role.team.value,
            "information": player.role.get_description()
        }

    def get_proposal_size(self) -> int:
        return _MISSION_NUM_TO_PROPOSAL_SIZE[self.get_num_players()][self.mission_num]

    def get_proposal_info(self) -> Dict[str, Any]:
        if self.lobby_status != LobbyStatus.IN_PROGRESS:
            raise ValueError("Can only get proposal info when game in progress")
        return {
            "proposal_order": self.proposal_order_names,
            "proposer_id": self.proposer_id,
            "proposer_index": self.proposer_index,
            "proposal_size": self.get_proposal_size(),
            "max_num_proposers": 2 if self.mission_num == 0 else self.max_num_proposers,
        }

    def get_round_info(self) -> Dict[str, Any]:
        def _get_special_mission_info():
            if self.mission_num == 0:
                return "The first mission has only two proposals. No voting will happen until both proposals are " \
                       "made. Upvote for the first proposal, downvote for the second proposal."
            elif self.mission_num == 3 and self.get_num_players() >= 7:
                return "There are two fails required for this mission to fail."
            return ""

        if self.lobby_status != LobbyStatus.IN_PROGRESS:
            raise ValueError("Can only get mission info when game in progress")
        return {
            "mission_num": self.mission_num,
            "mission_info": _get_special_mission_info()
        }

    def get_mission_info(self) -> Dict[str, Any]:
        # proposal index is the prpoosal going. Should always be 0 unless round 1 with downvotes
        return {
            "mission_players": self.current_mission,
            "mission_session_ids": [self.player_name_to_session_id[name] for name in self.current_mission],
        }

    def send_mission(self, proposal_idx: int) -> Dict[str, Any]:
        # handle updating mission info, then return call to get mission info
        self.current_proposal_num = 1 # reset for next round
        self.current_mission = self.current_proposals[proposal_idx]
        self.current_proposals = []
        return self.get_mission_info()

    def set_proposal(self, player_names: List[str]) -> Dict[str, Any]:
        if self.lobby_status != LobbyStatus.IN_PROGRESS:
            raise ValueError("Can only set proposal when game in progress")
        if self.game_phase != GamePhase.PROPOSAL:
            raise ValueError("Can only set proposals when game state is proposal")

        expected_proposal_size = self.get_proposal_size()
        if len(player_names) != expected_proposal_size:
            raise ValueError(f"Expected proposal of size {expected_proposal_size}, but instead got {player_names}.")
        for player_name in player_names:
            if player_name not in self.proposal_order_names:
                raise ValueError(f"{player_name} is not in the game.")

        def _advance_proposal():
            self.current_proposal_num += 1
            self.number_votes = 0  # reset number votes from prior round
            self.proposer_index = (self.proposer_index + 1) % self.get_num_players()
            self.proposer_id = self.proposal_order_players[self.proposer_index].session_id

        self.current_proposals.append(player_names)
        if len(self.current_proposals) == 1 and self.mission_num == 0:
            _advance_proposal()
            return {
                "game_phase": self.game_phase,
                "proposals": self.current_proposals,
                "proposal_info": self.get_proposal_info()
            }

        if (self.mission_num == 0 and len(self.current_proposals) != 2) or \
                (self.mission_num != 0 and len(self.current_proposals) != 1):
            raise ValueError("To enter voting phase, must be first mission with 2 proposals, or only have 1 proposal.")

        # if not mission 1, and current proposal num equals max num proposals, then this mission must go
        if self.mission_num != 0 and self.current_proposal_num == self.max_num_proposers:
            _advance_proposal()
            self.game_phase = GamePhase.MISSION
            return {
                "game_phase": self.game_phase,
                "mission_info": self.send_mission(0)
            }

        _advance_proposal() # advance proposal for next round
        self.game_phase = GamePhase.VOTE
        return {
            "game_phase": self.game_phase,
            "proposals": self.current_proposals
        }

    def set_vote(self, session_id: str, vote: bool) -> Dict[str, Any]:
        if self.lobby_status != LobbyStatus.IN_PROGRESS:
            raise ValueError("Can only set vote when game in progress")
        if self.game_phase != GamePhase.VOTE:
            raise ValueError("Can only set vote when game state is vote")

        player = self.session_id_to_player[session_id]
        if player.proposal_vote is not None:
            raise ValueError(f"{player.name} has already voted this round.")
        player.proposal_vote = vote
        self.number_votes += 1

        # if still waiting on other to vote, then just send back game phase and vote
        if self.number_votes != self.get_num_players():
            return {
                "game_phase": self.game_phase,
                "vote": vote
            }

        # everyone has voted, so process votes
        # First determine number of upvotes
        upvotes = 0
        for player in self.session_id_to_player.values():
            if player.proposal_vote:
                upvotes += 1

        # build up vote dictionary and clear votes for the future
        for player in self.session_id_to_player.values():
            self.last_vote_info[player.name] = player.proposal_vote
            player.proposal_vote = None

        # if upvote, send mission. Will always be index 0, even in round 1
        if upvotes > (self.get_num_players() / 2):
            self.game_phase = GamePhase.MISSION
            return {
                "game_phase": self.game_phase,
                "proposal_vote_info": self.last_vote_info,
                "mission_info": self.send_mission(0)
            }

        # downvotes on mission 1 indicate send second proposal
        if self.mission_num == 0:
            self.game_phase = GamePhase.MISSION
            return {
                "game_phase": self.game_phase,
                "proposal_vote_info": self.last_vote_info,
                "mission_info": self.send_mission(1)
            }

        # reset current proposals for next proposal
        self.current_proposals = []

        # else return next proposal info, which was updated by set_proposal
        self.game_phase = GamePhase.PROPOSAL
        return {
            "game_phase": self.game_phase,
            "proposal_vote_info": self.last_vote_info,
            "proposal_info": self.get_proposal_info()
        }

    def play_mission_card(self, session_id: str, mission_card: MissionCard) -> Dict[str, Any]:
        if self.lobby_status != LobbyStatus.IN_PROGRESS:
            raise ValueError("Can only set vote when game in progress")
        if self.game_phase != GamePhase.MISSION:
            raise ValueError("Can only play mission cards when game state is mission")

        player = self.session_id_to_player[session_id]
        if player.name not in self.current_mission:
            raise ValueError(f"{player.name} not on current mission, only {self.current_mission} are on the mission.")
        if player.mission_card is not None:
            raise ValueError(f"{player.name} has already played a mission card this round.")
        if not player.role.validate_mission_card(mission_card):
            raise ValueError(f"{player.name} is not allowed to play the card {mission_card}.")
        player.mission_card = mission_card
        self.current_mission_count += 1

        # if still waiting on mission cards
        if self.current_mission_count != len(self.current_mission):
            return {
                "game_phase": self.game_phase,
                "mission_card": mission_card
            }

        # get played cards by looking at what each player on mission has played
        players_on_mission = [self.session_id_to_player[self.player_name_to_session_id[player_name]]
                              for player_name in self.current_mission]
        played_cards = [player.mission_card for player in players_on_mission]

        mission_result = MissionResult.PASS
        num_fails = played_cards.count(MissionCard.FAIL)
        num_reverse = played_cards.count(MissionCard.REVERSE)

        if num_reverse == 2:
            num_reverse = 0
        # handle two fails separately
        if self.get_num_players() >= 7 and self.mission_num == 3:
            if (num_reverse == 1 and num_fails == 1) or (num_reverse == 0 and num_fails >= 2):
                mission_result = MissionResult.FAIL
        elif num_fails > 0:
            if num_reverse == 1:
                mission_result = MissionResult.PASS
            else:
                mission_result = MissionResult.FAIL
        elif num_reverse == 1:
            # at this point, no fails
            mission_result = MissionResult.FAIL

        self.mission_players[self.mission_num] = self.current_mission
        self.mission_num_to_result[self.mission_num] = mission_result
        num_failed_missions = list(self.mission_num_to_result.values()).count(MissionResult.FAIL)
        num_passed_missions = len(self.mission_num_to_result) - num_failed_missions

        # increment and clear mission cards for next round
        self.mission_num += 1
        for player in players_on_mission:
            player.mission_card = None
        self.current_mission_count = 0

        if num_passed_missions == 3:
            self.game_phase = GamePhase.ASSASSINATION
            return {
                "mission_result": mission_result,
                "game_phase": self.game_phase
            }
        if num_failed_missions == 3:
            self.lobby_status = LobbyStatus.DONE
            self.game_phase = GamePhase.DONE
            return {
                "mission_result": mission_result,
                "lobby_status": self.lobby_status,
                "game_phase": self.game_phase
            }

        self.game_phase = GamePhase.PROPOSAL
        return {
            "mission_result": mission_result,
            "game_phase": self.game_phase,
            "proposal_info": self.get_proposal_info()
        }
