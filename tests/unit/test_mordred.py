import pytest
from game.player import Player
from game.roles.mordred import Mordred
from unittest.mock import Mock, PropertyMock


def test_use_ability_fails():
    mordred = Mordred()
    with pytest.raises(ValueError):
        mordred.use_ability()


def test_get_description():
    mordred = Mordred()
    player1 = Player("session_id", "Evil")
    evil_mock = Mock()
    type(evil_mock).role_name = PropertyMock(return_value="Morgana")
    player1.role = evil_mock
    mordred.add_seen_player(player1)

    expected = "You are Mordred [EVIL].\n\nYou are hidden from all Good Information roles.\nLike other Evil " \
               "characters, you know who else is Evil (except Colgrevance).\n\nEvil is Evil."

    assert mordred.get_description() == expected
