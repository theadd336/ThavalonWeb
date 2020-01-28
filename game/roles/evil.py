from ..player import Player
from ..role import Role, Team
from typing import List


class Evil(Role):
    def __init__(self, role_name: str, team: Team, is_reverser: bool = False) -> None:
        self.saw_colgrevance: bool = False
        self.saw_titania: bool = False
        super().__init__(role_name, team, is_reverser)

    def add_seen_player(self, player: Player) -> bool:
        if player.role.role_name == "Colgrevance":
            self.saw_colgrevance = True
        elif player.role.role_name == "Titania":
            self.saw_titania = True
        if player.role.team == Team.EVIL and player.role.role_name != "Colgrevance":
            super().add_seen_player(player)
            return True
        return False

    def get_shared_description(self) -> str:
        result: List[str] = ["Like other Evil characters, you know who else is Evil (except Colgrevance).\n"]
        result += [f"{player.name} is Evil." for player in self.players_seen]
        if self.saw_colgrevance:
            result.append("Colgrevance lurks in the shadows. (There is another evil that you do not see).")
        if self.saw_titania:
            result.append("Titania has infiltrated your ranks. (One of the people is not Evil).")
        return "\n".join(result).strip()  # remove new line if there are no seen evil
