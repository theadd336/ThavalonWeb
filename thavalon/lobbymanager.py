from game.singleton import Singleton
import threading
from game.gamemanager import GameManager
from typing import Dict, List

_GAME_MANAGER = GameManager()


class LobbyManager(metaclass=Singleton):
    def __init__(self) -> None:
        self._count = 0
        self._lobby_to_game: Dict[str, str] = dict()
        self._game_to_lobby: Dict[str, str] = dict()
        # self._clear_empty_lobbies(threading.Event())

    def create_new_lobby(self, game_id: str):
        if self._game_to_lobby.get(game_id) is not None:
            raise ValueError("The lobby already exists.")

        self._count += 1
        self._lobby_to_game[str(self._count)] = game_id
        self._game_to_lobby[game_id] = str(self._count)
        return self._count

    def get_game_from_lobby(self, lobby_index: str) -> str:
        if type(lobby_index) != str:
            raise TypeError("Lobby index must be a string.")
        return self._lobby_to_game[lobby_index]

    def remove_lobby(self, game_id: str) -> None:
        lobby = self._game_to_lobby.get(game_id)
        if lobby is None:
            raise ValueError("The lobby no longer exists.")

        del self._game_to_lobby[game_id]
        del self._lobby_to_game[lobby]
        self._count -= 1

    def list_all_lobbies(self) -> List[str]:
        return [lobby_id for lobby_id in self._lobby_to_game.keys()]

    def _clear_empty_lobbies(self, stop_event):
        if not stop_event.is_set():
            threading.Timer(60, self._clear_empty_lobbies, [stop_event]).start()

        game_ids = [game_id for game_id in self._game_to_lobby.keys() if
                    _GAME_MANAGER.get_game(game_id).get_num_players() == 0]

        for game_id in game_ids:
            self.remove_lobby(game_id)
            _GAME_MANAGER.delete_game(game_id)
