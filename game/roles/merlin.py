from ..role import Role, Team


class Merlin(Role):
    def __init__(self):
        super().__init__("Merlin", Team.GOOD)

    def get_description(self):
        return "You are Merlin [GOOD].\n\n" \
            "You know which people have Evil roles, but not who has any specific role.\n" \
            "You are a valid Assassination target.\n" \
            "\n".join([f"You see {player.name} as evil." for player in self.players_seen])

    def use_ability(self):
        raise ValueError("Merlin does not have an ability to use.")
