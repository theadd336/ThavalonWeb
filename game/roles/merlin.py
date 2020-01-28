from ..player import Player
from ..role import Role, Team


class Merlin(Role):
    def __init__(self):
        super().__init__("Merlin", Team.GOOD)

    def get_description(self) -> str:
        return "\n".join([
            "You are Merlin [GOOD].\n",
            "You know which people have Evil roles, but not who has any specific role.",
            "You are a valid Assassination target.\n",
            *[f"You see {player.name} as evil." for player in self.players_seen]
        ])

    def add_seen_player(self, player: Player) -> bool:
        if (player.role.team == Team.EVIL and player.role.role_name != "Mordred") or \
                player.role.role_name == "Lancelot":
            super().add_seen_player(player)
            return True
        return False

    def use_ability(self) -> None:
        raise ValueError("Merlin does not have an ability to use.")
