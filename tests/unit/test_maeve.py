import pytest
from conftest import iseult, merlin, mordred, morgana, tristan
from game.roles.maeve import Maeve


def test_use_ability():
    maeve = Maeve()
    maeve.use_ability()
    maeve.use_ability()
    maeve.use_ability()
    with pytest.raises(ValueError):
        maeve.use_ability()


@pytest.mark.parametrize("is_assassin", [True, False])
def test_get_description(is_assassin):
    maeve = Maeve(is_assassin=is_assassin)
    maeve.add_seen_player(morgana)
    expected = "You are Maeve [EVIL].\n\nOnce per round (except the first), during a vote on a proposal, " \
               "you can secretly choose to obscure how\neach player voted on the proposal and instead have only the " \
               "amount of upvotes and downvotes presented.\nLike other Evil characters, you know who else is Evil (" \
               "except Colgrevance).\n\nMorgana is Evil."
    if is_assassin:
        expected += "\n\nYou are the assassin!"
    assert maeve.get_description() == expected


@pytest.mark.parametrize("player, expected", [
    (iseult, False),
    (merlin, False),
    (mordred, True),
    (morgana, True),
    (tristan, False)
])
def test_add_players(player, expected):
    maeve = Maeve()
    assert maeve.add_seen_player(player) == expected
