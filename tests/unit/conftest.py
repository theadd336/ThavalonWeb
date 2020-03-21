import os
import sys
from unittest.mock import Mock, PropertyMock

# add path to ThavalonWeb to sys.path, for tests to successfully import from game
sys.path.append(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from game.player import Player  # noqa
from game.roles.iseult import Iseult  # noqa
from game.roles.maeve import Maeve  # noqa
from game.roles.merlin import Merlin  # noqa
from game.roles.mordred import Mordred  # noqa
from game.roles.morgana import Morgana  # noqa
from game.roles.percival import Percival  # noqa
from game.roles.tristan import Tristan  # noqa

iseult = Player("iseult", "Iseult")
iseult.role = Iseult()
maeve = Player("maeve", "Maeve")
maeve.role = Maeve()
maeve.role.used_ability = Mock()
merlin = Player("merlin", "Merlin")
merlin.role = Merlin()
mordred = Player("mordred", "Mordred")
mordred.role = Mordred()
morgana = Player("morgana", "Morgana")
morgana.role = Morgana()
tristan = Player("tristan", "Tristan")
tristan.role = Tristan()
percival = Player("percival", "Percival")
percival.role = Percival()
generic_player = Player("generic", "Generic")
generic_player.role = Iseult()

# TODO: replace role mocks with actual roles when available
colgrevance = Player("colgrevance", "Colgrevance")
mock_colgrevance_role = Mock()
type(mock_colgrevance_role).role_name = PropertyMock(return_value="Colgrevance")
colgrevance.role = mock_colgrevance_role

titania = Player("titania", "Titania")
mock_titania_role = Mock()
type(mock_titania_role).role_name = PropertyMock(return_value="Titania")
titania.role = mock_titania_role
