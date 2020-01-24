from .evil import Evil
from ..role import Team


class Morgana(Evil):
    def __init__(self):
        super().__init__("Morgana", Team.EVIL)

    def get_description(self):
        return "\n".join([
            "You are Morgana [EVIL].\n",
            "You appear like Merlin to Percival.",
            self.get_shared_description()
        ])

    def use_ability(self):
        raise ValueError("Morgana does not have an ability to use.")
