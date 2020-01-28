import pytest
from conftest import morgana, mordred, iseult, tristan
from game.roles.merlin import Merlin


def test_use_ability_fails():
    merlin = Merlin()
    with pytest.raises(ValueError):
        merlin.use_ability()


def test_get_description():
    merlin = Merlin()
    merlin.add_seen_player(morgana)
    merlin.add_seen_player(mordred)

    expected = "You are Merlin [GOOD].\n\nYou know which people have Evil roles, but not who has any specific " \
               "role.\nYou are a valid Assassination target.\n\nYou see Morgana as evil."

    assert merlin.get_description() == expected


@pytest.mark.parametrize("player, expected", [
    (iseult, False),
    (mordred, False),
    (morgana, True),
    (tristan, False)
])
def test_add_players(player, expected):
    merlin = Merlin()
    assert merlin.add_seen_player(player) == expected
