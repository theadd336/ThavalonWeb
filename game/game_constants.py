from enum import Enum


class GamePhase(Enum):
    PROPOSAL = 0
    VOTE = 1
    MISSION = 2
    ASSASSINATION = 3


class LobbyStatus(Enum):
    JOINING = 0
    IN_PROGRESS = 1
    DONE = 2


class MissionCard(Enum):
    SUCCESS = 0
    FAIL = 1
    REVERSE = 2


class MissionResult(Enum):
    PASS = 0
    FAIL = 1