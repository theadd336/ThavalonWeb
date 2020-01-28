import pytest
from conftest import iseult, merlin, mordred, tristan
from game.roles.morgana import Morgana


def test_use_ability_fails():
    morgana = Morgana()
    with pytest.raises(ValueError):
        morgana.use_ability()


def test_get_description():
    morgana = Morgana()
    morgana.add_seen_player(mordred)
    expected = "You are Morgana [EVIL].\n\nYou appear like Merlin to Percival.\nLike other Evil " \
               "characters, you know who else is Evil (except Colgrevance).\n\nMordred is Evil."

    assert morgana.get_description() == expected


@pytest.mark.parametrize("player, expected", [
    (iseult, False),
    (merlin, False),
    (mordred, True),
    (tristan, False)
])
def test_add_players(player, expected):
    morgana = Morgana()
    assert morgana.add_seen_player(player) == expected
