from abc import ABC
from enum import Enum


class Team(Enum):
    EVIL = 0
    GOOD = 1


class Role(ABC):
    def __init__(self, role_name: str, team: Team, is_reverser: bool = False) -> None:
        self.role_name = role_name
        self.team = team
        self.is_reverser = is_reverser

