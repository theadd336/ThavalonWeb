from .game_constants import MissionCard, Team
from .player import Player
from abc import ABC, abstractmethod
from typing import List


class Role(ABC):
    def __init__(self, role_name: str, team: Team, is_reverser: bool = False, is_assassin: bool = False) -> None:
        self.role_name = role_name
        self.team = team
        self.is_reverser = is_reverser
        self.players_seen: List[Player] = []
        self.used_ability: bool = True
        self.is_assassin = is_assassin

    @abstractmethod
    def get_description(self) -> str:
        pass

    @abstractmethod
    def use_ability(self) -> None:
        pass

    @abstractmethod
    def add_seen_player(self, player: Player) -> bool:
        self.players_seen.append(player)

    # TODO: Test
    def validate_mission_card(self, card: MissionCard) -> bool:
        # only reversers can reverse
        if card == MissionCard.REVERSE:
            return self.is_reverser
        # only evil is allowed to fail
        if card == MissionCard.FAIL:
            return self.team == Team.EVIL
        # successes are always allowed
        return True

    # TODO: Test
    def __eq__(self, other):
        if not isinstance(other, self.__class__):
            return False
        return self.role_name == other.role_name

    # TODO: Test
    def __ne__(self, other):
        return not self.__eq__(other)
