import pytest
from conftest import iseult, merlin, mordred, morgana, tristan
from game.roles.maelegant import Maelegant


def test_use_ability_fails():
    maelegant = Maelegant()
    with pytest.raises(ValueError):
        maelegant.use_ability()


@pytest.mark.parametrize("is_assassin", [True, False])
def test_get_description(is_assassin):
    maelegant = Maelegant(is_assassin=is_assassin)
    maelegant.add_seen_player(morgana)
    expected = "You are Maelegant [EVIL].\n\nYou may play Reversal cards while on missions.\nLike other Evil " \
               "characters, you know who else is Evil (except Colgrevance).\n\nMorgana is Evil."
    if is_assassin:
        expected += "\n\nYou are the assassin!"
    assert maelegant.get_description() == expected


@pytest.mark.parametrize("player, expected", [
    (iseult, False),
    (merlin, False),
    (mordred, True),
    (morgana, True),
    (tristan, False)
])
def test_add_players(player, expected):
    maelegant = Maelegant()
    assert maelegant.add_seen_player(player) == expected
