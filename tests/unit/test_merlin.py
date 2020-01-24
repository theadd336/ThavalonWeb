import pytest
from game.player import Player
from game.roles.merlin import Merlin


def test_use_ability_fails():
    merlin = Merlin()
    with pytest.raises(ValueError):
        merlin.use_ability()


def test_get_description():
    merlin = Merlin()
    player1 = Player("session_id", "Meg")
    player2 = Player("session_id2", "Andrew")
    merlin.add_seen_player(player1)
    merlin.add_seen_player(player2)

    expected = "You are Merlin [GOOD].\n\nYou know which people have Evil roles, but not who has any specific " \
               "role.\nYou are a valid Assassination target.\n\nYou see Meg as evil.\nYou see Andrew as evil."

    assert merlin.get_description() == expected
