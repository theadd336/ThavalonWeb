import uuid
from .game import Game
from .singleton import Singleton
from typing import Tuple

class GameManager(metaclass=Singleton):
    def __init__(self):
        self.uuid_to_game = {}

    def create_new_game(self) -> Tuple[str, Game]:
        game_uuid = str(uuid.uuid4())
        game = Game()
        if game_uuid in self.uuid_to_game:
            raise ValueError("Unable to make game, game uuid already exists.")
        self.uuid_to_game[game_uuid] = game
        return game_uuid

    def get_game(self, game_uuid: str) -> Game:
        if game_uuid not in self.uuid_to_game:
            raise ValueError(f"UUID {game_uuid} is not a valid game uuid.")
        return self.uuid_to_game[game_uuid]

    def delete_game(self, game_uuid: str) -> None:
        if game_uuid not in self.uuid_to_game:
            raise ValueError(f"UUID {game_uuid} is not a valid game uuid.")
        del self.uuid_to_game[game_uuid]