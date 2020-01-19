from .playermanager import PlayerManager

MAX_NUM_PLAYERS = 10


class Game:
    def __init__(self) -> None:
        self.player_manager = PlayerManager()

    def get_num_players(self) -> int:
        return len(self.player_manager.session_id_to_player)

    def is_game_full(self) -> bool:
        return self.get_num_players() == MAX_NUM_PLAYERS

    def add_player(self, session_id: str, player_name: str) -> None:
        if self.is_game_full():
            raise ValueError(f"Game currently has max {MAX_NUM_PLAYERS} players, cannot add new player.")
        self.player_manager.add_player(session_id, player_name)

    def remove_player(self, session_id: str) -> None:
        self.player_manager.remove_player(session_id)