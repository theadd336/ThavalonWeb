import random
from .player import Player
from enum import Enum
from typing import Any, Dict, List

_MIN_NUM_PLAYERS = 5
_MAX_NUM_PLAYERS = 10

_MISSION_SIZE_TO_PROPOSAL_SIZE = {
    5: [2, 3, 2, 3, 3],
    6: [2, 3, 4, 3, 4],
    7: [2, 3, 3, 4, 4],
    8: [3, 4, 4, 5, 5],
    9: [3, 4, 4, 5, 5],
    10: [3, 4, 4, 5, 5]
}


class Vote(Enum):
    SUCCESS = 0
    FAIL = 1
    REVERSE = 2


class GameState(Enum):
    IN_LOBBY = 0
    IN_PROGRESS = 1
    DONE = 2


class Game:
    def __init__(self) -> None:
        # Main Game Info
        # map session id to actual player object
        self.session_id_to_player: Dict[str, Player] = {}
        # map player name to session_id
        self.player_name_to_session_id: Dict[str, str] = {}
        # current state of game
        self.game_state: GameState = GameState.IN_LOBBY

        # Information about proposals
        # the proposal, or seating, order of players
        self.proposal_order: List[str] = []
        # the index number of the current proposer, 0-indexed
        self.proposer_idx: int = 0
        # the current proposal number, 1-indexed.
        self.proposal_num: int = 1

        # the current mission number, 0-indexed.
        self.mission_num: int = 0

    # methods for adding players
    def get_num_players(self) -> int:
        return len(self.session_id_to_player)

    def is_game_full(self) -> bool:
        return self.get_num_players() == _MAX_NUM_PLAYERS

    def get_player_names_in_game(self) -> List[str]:
        return [player.name for player in self.session_id_to_player.values()]

    def add_player(self, session_id: str, name: str) -> List[str]:
        if self.game_state != GameState.IN_LOBBY:
            raise ValueError("Can only add player while in game.")
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
        return "session_id" in self.session_id_to_player

    def remove_player(self, session_id: str) -> List[str]:
        if self.game_state != GameState.IN_LOBBY:
            raise ValueError("Can only remove player while in lobby.")
        if session_id not in self.session_id_to_player:
            raise ValueError(f"Player with session id {session_id} does not exist")
        player_name = self.session_id_to_player[session_id].name
        del self.player_name_to_session_id[player_name]
        del self.session_id_to_player[session_id]
        return [player.name for player in self.session_id_to_player.values()]

    def get_starting_info(self, session_id: str) -> Dict[str, Any]:
        if session_id not in self.session_id_to_player:
            raise ValueError(f"Player with session id {session_id} does not exist")
        return {
            "player_role": self.session_id_to_player[session_id].role,
            "proposal_order": self.proposal_order,
            "first_proposer": self.proposal_order[-2],
            "second_proposer": self.proposal_order[-1],
            "num_on_mission": _MISSION_SIZE_TO_PROPOSAL_SIZE[self.get_num_players()][self.mission_num]
        }

    # method for starting the game
    def start_game(self) -> None:
        # validate players
        num_players = self.get_num_players()
        if num_players < _MIN_NUM_PLAYERS:
            raise ValueError(f"Game must have at least {_MIN_NUM_PLAYERS} to be started")
        if num_players > _MAX_NUM_PLAYERS:
            raise ValueError(f"Game somehow has more than {_MAX_NUM_PLAYERS}, unable to start")

        # generate seating order
        self.proposal_order = [player.name for player in self.session_id_to_player.values()]
        random.shuffle(self.proposal_order)
