from ..role import Role, Team


class Merlin(Role):
    def __init__(self):
        super().__init__("Merlin", Team.GOOD)

    def get_description(self):
        return "\n".join([
            "You are Merlin [GOOD].\n",
            "You know which people have Evil roles, but not who has any specific role.",
            "You are a valid Assassination target.\n",
            *[f"You see {player.name} as evil." for player in self.players_seen]
        ])

    def use_ability(self):
        raise ValueError("Merlin does not have an ability to use.")
