from .player import Player


class PlayerManager:
    def __init__(self) -> None:
        self.session_id_to_player = {}

    def add_player(self, session_id: str, name: str) -> Player:
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
