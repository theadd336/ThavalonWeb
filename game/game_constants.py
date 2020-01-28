from enum import Enum


class GameState(Enum):
    IN_LOBBY = 0
    IN_PROGRESS = 1
    DONE = 2


class Vote(Enum):
    SUCCESS = 0
    FAIL = 1
    REVERSE = 2
