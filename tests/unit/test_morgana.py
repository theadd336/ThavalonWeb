import pytest
from conftest import iseult, merlin, mordred, tristan
from game.roles.morgana import Morgana


def test_use_ability_fails():
    morgana = Morgana()
    with pytest.raises(ValueError):
        morgana.use_ability()


@pytest.mark.parametrize("is_assassin", [True, False])
def test_get_description(is_assassin):
    morgana = Morgana(is_assassin=is_assassin)
    morgana.add_seen_player(mordred)
    expected = "You are Morgana [EVIL].\n\nYou appear like Merlin to Percival.\nLike other Evil " \
               "characters, you know who else is Evil (except Colgrevance).\n\nMordred is Evil."
    if is_assassin:
        expected += "\n\nYou are the assassin!"
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
