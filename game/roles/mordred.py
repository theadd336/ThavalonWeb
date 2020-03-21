from .evil import Evil
from ..role import Team


class Mordred(Evil):
    def __init__(self, is_assassin=False):
        super().__init__("Mordred", Team.EVIL, is_assassin=is_assassin)

    def get_description(self):
        return "\n".join([
            "You are Mordred [EVIL].\n",
            "You are hidden from all Good Information roles.",
            self.get_shared_description()
        ])

    def use_ability(self):
        raise ValueError("Mordred does not have an ability to use.")
