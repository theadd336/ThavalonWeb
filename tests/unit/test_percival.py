import pytest
from conftest import iseult, merlin, morgana, tristan
from game.roles.percival import Percival


def test_use_ability_fails():
    percival = Percival()
    with pytest.raises(ValueError):
        percival.use_ability()


def test_get_description():
    percival = Percival()
    percival.add_seen_player(merlin)
    percival.add_seen_player(morgana)
    expected = "You are Percival [GOOD].\n\nYou know which people have the Merlin or Morgana roles, " \
               "but not specifically who has each.\nMerlin is Merlin or Morgana.\nMorgana is Merlin or Morgana."
    assert percival.get_description() == expected


@pytest.mark.parametrize("player, expected", [
    (iseult, False),
    (merlin, True),
    (morgana, True),
    (tristan, False)
])
def test_add_players(player, expected):
    percival = Percival()
    assert percival.add_seen_player(player) == expected
