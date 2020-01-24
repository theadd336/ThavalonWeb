import pytest
from game.player import Player
from game.roles.iseult import Iseult


def test_use_ability_fails():
    iseult = Iseult()
    with pytest.raises(ValueError):
        iseult.use_ability()


def test_get_description():
    iseult = Iseult()
    player = Player("session_id", "Test")
    iseult.add_seen_player(player)

    expected = "You are Iseult [GOOD].\n\nThe person you see is also Good and is aware that you are Good.\n" \
               "You and Tristan are collectively a valid Assassination target.\n\nTest is Tristan."

    assert iseult.get_description() == expected

def test_add_two_players_fails():
    iseult = Iseult()
    player = Player("session_id", "Test")
    iseult.add_seen_player(player)
    with pytest.raises(ValueError):
        player2 = Player("session_id2", "Name")
        iseult.add_seen_player(player2)
