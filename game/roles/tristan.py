from ..role import Player, Role, Team


class Tristan(Role):
    def __init__(self):
        super().__init__("Tristan", Team.GOOD)

    def get_description(self):
        return "\n".join([
            "You are Tristan [GOOD].\n",
            "The person you see is also Good and is aware that you are Good.",
            "You and Iseult are collectively a valid Assassination target.\n",
            f"{self.players_seen[0].name} is Iseult."
        ])

    def use_ability(self):
        raise ValueError("Tristan does not have an ability to use.")

    def add_seen_player(self, player: Player) -> None:
        if len(self.players_seen) == 1:
            raise ValueError("Tristan can see at most one other person.")
        self.players_seen.append(player)