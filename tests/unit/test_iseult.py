import pytest
from game.roles.iseult import Iseult
from conftest import morgana, merlin, mordred, tristan


def test_use_ability_fails():
    iseult = Iseult()
    with pytest.raises(ValueError):
        iseult.use_ability()


def test_get_description():
    iseult = Iseult()
    iseult.add_seen_player(tristan)
    expected = "You are Iseult [GOOD].\n\nThe person you see is also Good and is aware that you are Good.\n" \
               "You and Tristan are collectively a valid Assassination target.\n\nTristan is Tristan."

    assert iseult.get_description() == expected


@pytest.mark.parametrize("player, expected", [
    (morgana, False),
    (merlin, False),
    (mordred, False),
    (tristan, True)
])
def test_add_players(player, expected):
    iseult = Iseult()
    assert iseult.add_seen_player(player) == expected
