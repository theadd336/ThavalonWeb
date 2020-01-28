import pytest
from conftest import colgrevance, titania
from game.player import Player
from game.role import Team
from game.roles.evil import Evil
from unittest.mock import Mock, PropertyMock


@pytest.mark.parametrize("see_colgrevance, see_titania, expected_string", [
    (False, False, ""),
    (False, True, "\n\nTitania is Evil.\nTitania has infiltrated your ranks. (One of the people is not Evil)."),
    (True, False, "\n\nColgrevance lurks in the shadows. (There is another evil that you do not see)."),
    (True, True, "\n\nTitania is Evil.\nColgrevance lurks in the shadows. (There is another evil that you do not "
                 "see).\nTitania has infiltrated your ranks. (One of the people is not Evil).")
])
def test_seeing_colgrevance_and_titania(see_colgrevance, see_titania, expected_string):
    Evil.__abstractmethods__= set()
    evil = Evil("Evil", Team.EVIL)
    if see_colgrevance:
        evil.add_seen_player(colgrevance)
    if see_titania:
        evil.add_seen_player(titania)
    expected_string = "Like other Evil characters, you know who else is Evil (except Colgrevance)." + expected_string
    assert evil.get_shared_description() == expected_string
