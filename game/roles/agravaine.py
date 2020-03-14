from .evil import Evil
from ..role import Team
from game.game_constants import MissionCard

_NUM_USES = 3


class Agravaine(Evil):
    def __init__(self, is_assassin=False):
        self.ability_count = 0
            super().__init__("Agravaine", Team.EVIL, is_assassin=is_assassin)

    def get_description(self):
        return "\n".join([
            "You are Agravaine [EVIL].\n",
            "You may declare to fail a mission that you were on and would otherwise have succeeded.",
            self.get_shared_description()
        ])

    def validate_mission_card(self, card: MissionCard) -> bool:
        return card == MissionCard.FAIL

    def use_ability(self) -> None:
        pass
