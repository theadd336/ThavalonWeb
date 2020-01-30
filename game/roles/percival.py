from ..player import Player
from ..role import Role, Team


class Percival(Role):
    def __init__(self):
        super().__init__("Percival", Team.GOOD)

    def get_description(self) -> str:
        return "\n".join([
            "You are Percival [GOOD].\n",
            "You know which people have the Merlin or Morgana roles, but not specifically who has each.",
            *[f"{player.name} is Merlin or Morgana." for player in self.players_seen]
        ])

    def add_seen_player(self, player: Player) -> bool:
        if player.role.role_name == "Morgana" or player.role.role_name == "Merlin":
            super().add_seen_player(player)
            return True
        return False

    def use_ability(self) -> None:
        raise ValueError("Percival does not have an ability to use.")
