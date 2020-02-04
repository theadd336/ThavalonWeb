from .evil import Evil
from ..role import Team

_NUM_USES = 3


class Maeve(Evil):
    def __init__(self):
        self.used_ability = False
        self.ability_count = 0
        super().__init__("Maeve", Team.EVIL)

    def get_description(self):
        return "\n".join([
            "You are Maeve [EVIL].\n",
            "Once per round (except the first), during a vote on a proposal, you can secretly choose to obscure how",
            "each player voted on the proposal and instead have only the amount of upvotes and downvotes presented.",
            self.get_shared_description()
        ])

    def use_ability(self):
        if self.ability_count >= _NUM_USES:
            raise ValueError(f"You have already used your ability max {_NUM_USES} times.")
        self.used_ability = True
        self.ability_count += 1