from .lover import Lover
from ..player import Player


class Iseult(Lover):
    def __init__(self) -> None:
        super().__init__("Iseult", "Tristan")

    def add_seen_player(self, player: Player) -> bool:
        if player.role.role_name == "Tristan":
            super().add_seen_player(player)
            return True
        return False
