from .game import Vote
from .player import Player
from abc import ABC, abstractmethod
from enum import Enum
from typing import List


class Team(Enum):
    EVIL = 0
    GOOD = 1


class Role(ABC):
    def __init__(self, role_name: str, team: Team, is_reverser: bool = False) -> None:
        self.role_name = role_name
        self.team = team
        self.is_reverser = is_reverser
        self.players_seen: List[Player] = []

    @abstractmethod
    def get_description(self) -> str:
        pass

    @abstractmethod
    def use_ability(self) -> None:
        pass

    def add_seen_player(self, player: Player) -> None:
        self.players_seen.append(player)

    # TODO: Override for Agravaine to always return Fail
    def _validate_vote(self, vote: Vote) -> bool:
        # only reversers can reverse
        if vote == Vote.REVERSE:
            return self.is_reverser
        # only evil is allowed to fail
        if vote == Vote.FAIL:
            return self.team == Team.EVIL
        # successes are always allowed
        return True


