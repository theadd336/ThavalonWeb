import pytest
from conftest import iseult, merlin, morgana, tristan
from game.roles.mordred import Mordred


def test_use_ability_fails():
    mordred = Mordred()
    with pytest.raises(ValueError):
        mordred.use_ability()


def test_get_description():
    mordred = Mordred()
    mordred.add_seen_player(morgana)
    expected = "You are Mordred [EVIL].\n\nYou are hidden from all Good Information roles.\nLike other Evil " \
               "characters, you know who else is Evil (except Colgrevance).\n\nMorgana is Evil."
    assert mordred.get_description() == expected


@pytest.mark.parametrize("player, expected", [
    (iseult, False),
    (merlin, False),
    (morgana, True),
    (tristan, False)
])
def test_add_players(player, expected):
    mordred = Mordred()
    assert mordred.add_seen_player(player) == expected
