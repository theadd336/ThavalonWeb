from game.singleton import Singleton
from typing import Dict, Union


class LobbyManager(metaclass=Singleton):
    def __init__(self) -> None:
        self._count = 0
        self._lobby_to_game: Dict[str, str] = dict()
        self._game_to_lobby: Dict[str, str] = dict()

    def create_new_lobby(self, game_id: str):
        if self._game_to_lobby.get(game_id) is not None:
            raise ValueError("The lobby already exists.")

        self._count += 1
        self._lobby_to_game[str(self._count)] = game_id
        self._game_to_lobby[game_id] = str(self._count)
        return self._count

    def get_game_from_lobby(self, lobby_index: str) -> str:
        if type(lobby_index) != type(str):
            raise TypeError("Lobby index must be a string.")
        return self._lobby_to_game[lobby_index]

    def remove_lobby(self, game_id: str) -> None:
        lobby = self._game_to_lobby.get(game_id)
        if lobby is None:
            raise ValueError("The lobby no longer exists.")

        del self._game_to_lobby[game_id]
        del self._lobby_to_game[lobby]
        self._count -= 1
