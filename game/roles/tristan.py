from .lover import Lover
from ..player import Player


class Tristan(Lover):
    def __init__(self):
        super().__init__("Tristan", "Iseult")

    def add_seen_player(self, player: Player) -> bool:
        if player.role.role_name == "Iseult":
            super().add_seen_player(player)
            return True
        return False
