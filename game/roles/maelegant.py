from .evil import Evil
from ..role import Team


class Maelegant(Evil):
    def __init__(self, is_assassin=False):
        super().__init__("Maelegant", Team.EVIL, is_reverser=True, is_assassin=is_assassin)

    def get_description(self) -> str:
        return "\n".join([
            "You are Maelegant [EVIL].\n",
            "You may play Reversal cards while on missions.",
            self.get_shared_description()
        ])

    def use_ability(self) -> None:
        raise ValueError("Maelegant does not have an ability to use.")
