import pytest
from game.roles.tristan import Tristan
from conftest import iseult, morgana, merlin, mordred


def test_use_ability_fails():
    tristan = Tristan()
    with pytest.raises(ValueError):
        tristan.use_ability()


def test_get_description():
    tristan = Tristan()
    tristan.add_seen_player(iseult)
    expected = "You are Tristan [GOOD].\n\nThe person you see is also Good and is aware that you are Good.\n" \
               "You and Iseult are collectively a valid Assassination target.\n\nIseult is Iseult."

    assert tristan.get_description() == expected


@pytest.mark.parametrize("player, expected", [
    (morgana, False),
    (merlin, False),
    (mordred, False),
    (iseult, True)
])
def test_add_players(player, expected):
    tristan = Tristan()
    assert tristan.add_seen_player(player) == expected
