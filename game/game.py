from .player import Player

MAX_NUM_PLAYERS = 10


class Game:
    def __init__(self) -> None:
        self.session_id_to_player = {}

    def get_num_players(self) -> int:
        return len(self.session_id_to_player)

    def is_game_full(self) -> bool:
        return self.get_num_players() == MAX_NUM_PLAYERS

    def add_player(self, session_id: str, name: str) -> None:
        if self.is_game_full():
            raise ValueError(f"Game currently has max {MAX_NUM_PLAYERS} players, cannot add new player.")
        if session_id in self.session_id_to_player:
            raise ValueError(f"Session id {session_id} already in playermanager.")
        if name in [player.name for player in self.session_id_to_player.values()]:
            raise ValueError(f"Player with name {name} already in game.")
        player = Player(name)
        self.session_id_to_player[session_id] = player
        return player

    def get_player(self, session_id: str) -> Player:
        if session_id not in self.session_id_to_player:
            raise ValueError(f"Player with session id {session_id} does not exist")
        return self.session_id_to_player[session_id]

    def remove_player(self, session_id: str) -> None:
        if session_id not in self.session_id_to_player:
            raise ValueError(f"Player with session id {session_id} does not exist")
        del self.session_id_to_player[session_id]
