import os
import sys
from mock import Mock, PropertyMock

# add path to ThavalonWeb to sys.path, for tests to successfully import from game
sys.path.append(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from game.player import Player
from game.roles.iseult import Iseult
from game.roles.merlin import Merlin
from game.roles.mordred import Mordred
from game.roles.morgana import Morgana
from game.roles.tristan import Tristan

iseult = Player("iseult", "Iseult")
iseult.role = Iseult()
merlin = Player("merlin", "Merlin")
merlin.role = Merlin()
mordred = Player("mordred", "Mordred")
mordred.role = Mordred()
morgana = Player("morgana", "Morgana")
morgana.role = Morgana()
tristan = Player("tristan", "Tristan")
tristan.role = Tristan()

# TODO: replace role mocks with actual roles when available
colgrevance = Player("colgrevance", "Colgrevance")
mock_colgrevance_role = Mock()
type(mock_colgrevance_role).role_name = PropertyMock(return_value="Colgrevance")
colgrevance.role = mock_colgrevance_role

titania = Player("titania", "Titania")
mock_titania_role = Mock()
type(mock_titania_role).role_name = PropertyMock(return_value="Titania")
titania.role = mock_titania_role
