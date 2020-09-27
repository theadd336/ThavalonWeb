from ..player import Player
from ..role import Role, Team


class Nimue(Role):
    def __init__(self):
        super().__init__("Nimue", Team.GOOD)

    def get_description(self) -> str:
        return "\n".join([
            "You are Nimue [GOOD].\n",
            "You know which Good and Evil roles are in the game, but not who has any given role.",
            "You are a valid Assassination target.\n",
            "The following roles are in the game:",
            *[f"{player.role.role_name}" for player in self.players_seen]
        ])

    def add_seen_player(self, player: Player) -> bool:
        super().add_seen_player(player)
        return True

    def use_ability(self) -> None:
        raise ValueError("Nimue does not have an ability to use.")
