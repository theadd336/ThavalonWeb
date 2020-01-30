import random
from .player import Player
from game.roles.iseult import Iseult
from game.roles.lancelot import Lancelot
from game.roles.merlin import Merlin
from game.roles.mordred import Mordred
from game.roles.morgana import Morgana
from game.roles.percival import Percival
from game.roles.tristan import Tristan
from game.game_constants import GameState
from typing import Any, Dict, List

_MIN_NUM_PLAYERS = 1
_MAX_NUM_PLAYERS = 10

_MISSION_SIZE_TO_PROPOSAL_SIZE = {
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
_EVIL_ROLES = [Mordred, Morgana]


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

        # shuffle player in order
        players = list(self.session_id_to_player.values())
        random.shuffle(players)

        # proposal order is player names shuffled
        self.proposal_order = [player.name for player in players]
        random.shuffle(self.proposal_order)

        # get number good/evil in game
        num_good = _GAME_SIZE_TO_GOOD_COUNT[num_players]
        num_evil = num_players - num_good

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
                lover_roles_not_in_game = list(set(_GOOD_ROLES) - set(good_roles_in_game) - set([Iseult, Tristan]))
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
            for other_player in players[index+1:]:
                if player != other_player:
                    player.role.add_seen_player(other_player)
                    other_player.role.add_seen_player(player)

        self.game_state = GameState.IN_PROGRESS
