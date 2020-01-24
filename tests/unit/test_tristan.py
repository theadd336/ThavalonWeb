import pytest
from game.player import Player
from game.roles.tristan import Tristan


def test_use_ability_fails():
    tristan = Tristan()
    with pytest.raises(ValueError):
        tristan.use_ability()


def test_get_description():
    tristan = Tristan()
    player = Player("session_id", "Test")
    tristan.add_seen_player(player)

    expected = "You are Tristan [GOOD].\n\nThe person you see is also Good and is aware that you are Good.\n" \
               "You and Iseult are collectively a valid Assassination target.\n\nTest is Iseult."

    assert tristan.get_description() == expected


def test_add_two_players_fails():
    tristan = Tristan()
    player = Player("session_id", "Test")
    tristan.add_seen_player(player)
    with pytest.raises(ValueError):
        player2 = Player("session_id2", "Name")
        tristan.add_seen_player(player2)
