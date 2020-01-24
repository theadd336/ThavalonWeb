import pytest
from game.player import Player
from game.roles.morgana import Morgana
from unittest.mock import Mock, PropertyMock


def test_use_ability_fails():
    morgana = Morgana()
    with pytest.raises(ValueError):
        morgana.use_ability()


def test_get_description():
    morgana = Morgana()
    player1 = Player("session_id", "Evil")
    evil_mock = Mock()
    type(evil_mock).role_name = PropertyMock(return_value="Mordred")
    player1.role = evil_mock
    morgana.add_seen_player(player1)

    expected = "You are Morgana [EVIL].\n\nYou appear like Merlin to Percival.\nLike other Evil " \
               "characters, you know who else is Evil (except Colgrevance).\n\nEvil is Evil."

    assert morgana.get_description() == expected
