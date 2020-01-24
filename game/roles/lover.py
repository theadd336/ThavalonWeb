from ..role import Player, Role, Team


class Lover(Role):
    def __init__(self, role_name: str, lover_name: str) -> None:
        self.role_name = role_name
        self.lover_name = lover_name
        super().__init__(self.role_name, Team.GOOD)

    def get_description(self) -> str:
        return "\n".join([
            f"You are {self.role_name} [GOOD].\n",
            "The person you see is also Good and is aware that you are Good.",
            f"You and {self.lover_name} are collectively a valid Assassination target.\n",
            f"{self.players_seen[0].name} is {self.lover_name}."
        ])

    def use_ability(self):
        raise ValueError(f"{self.role_name} does not have an ability to use.")

    def add_seen_player(self, player: Player) -> None:
        if len(self.players_seen) == 1:
            raise ValueError(f"{self.role_name} can see at most one other person.")
        super().add_seen_player(player)