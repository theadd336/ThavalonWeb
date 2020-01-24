from ..role import Player, Role, Team


class Iseult(Role):
    def __init__(self):
        super().__init__("Iseult", Team.GOOD)

    def get_description(self):
        return "\n".join([
            "You are Iseult [GOOD].\n",
            "The person you see is also Good and is aware that you are Good.",
            "You and Tristan are collectively a valid Assassination target.\n",
            f"{self.players_seen[0].name} is Tristan."
        ])

    def use_ability(self):
        raise ValueError("Iseult does not have an ability to use.")

    def add_seen_player(self, player: Player) -> None:
        if len(self.players_seen) == 1:
            raise ValueError("Iseult can see at most one other person.")
        super().add_seen_player(player)