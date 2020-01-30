from ..player import Player
from ..role import Role, Team


class Lancelot(Role):
    def __init__(self):
        super().__init__("Lancelot", Team.GOOD)

    def get_description(self) -> str:
        return "\n".join([
            "You are Lancelot [GOOD].",
            "You may play Reversal cards while on missions.",
            "You appear Evil to Merlin."
        ])

    def add_seen_player(self, player: Player) -> bool:
        return False

    def use_ability(self) -> None:
        raise ValueError("Lancelot does not have an ability to use.")
