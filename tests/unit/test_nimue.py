import pytest
from conftest import iseult, merlin, morgana, tristan
from game.roles.nimue import Nimue


def test_use_ability_fails():
    nimue = Nimue()
    with pytest.raises(ValueError):
        nimue.use_ability()


def test_get_description():
    nimue = Nimue()
    nimue.add_seen_player(iseult)
    nimue.add_seen_player(morgana)
    expected = "You are Nimue [GOOD].\n\nYou know which Good and Evil roles are in the game, but not who has any given " \
               "role.\nYou appear Evil to Merlin.\nYou are a valid Assassination target.\n\nThe following roles are " \
               "in the game:\nIseult\nMorgana"
    assert nimue.get_description() == expected


@pytest.mark.parametrize("player, expected", [
    (iseult, True),
    (merlin, True),
    (morgana, True),
    (tristan, True)
])
def test_add_players(player, expected):
    nimue = Nimue()
    assert nimue.add_seen_player(player) == expected
