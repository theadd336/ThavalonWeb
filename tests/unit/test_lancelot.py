import pytest
from conftest import iseult, merlin, morgana, tristan
from game.roles.lancelot import Lancelot


def test_use_ability_fails():
    lancelot = Lancelot()
    with pytest.raises(ValueError):
        lancelot.use_ability()


def test_get_description():
    lancelot = Lancelot()
    expected = "You are Lancelot [GOOD].\nYou may play Reversal cards while on missions.\nYou appear Evil to Merlin."
    assert lancelot.get_description() == expected


@pytest.mark.parametrize("player, expected", [
    (iseult, False),
    (merlin, False),
    (morgana, False),
    (tristan, False)
])
def test_add_players(player, expected):
    lancelot = Lancelot()
    assert lancelot.add_seen_player(player) == expected
