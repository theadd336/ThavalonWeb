from .evil import Evil
from ..role import Team

class Maeve(Evil):
    def __init__(self, is_assassin=False):
        self.ability_count = 0
        self.used_ability = False
        super().__init__("Maeve", Team.EVIL, is_assassin=is_assassin)

    def get_description(self):
        return "\n".join([
            "You are Maeve [EVIL].\n",
            "Once per round (except the first), during a vote on a proposal, you can secretly choose to obscure how",
            "each player voted on the proposal and instead have only the amount of upvotes and downvotes presented.",
            self.get_shared_description()
        ])
