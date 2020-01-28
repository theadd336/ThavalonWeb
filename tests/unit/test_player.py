import pytest
from game.player import Player


@pytest.mark.parametrize("player1, player2, expected", [
    (Player("session_id", "name"), Player("session_id", "name"), True),
    (Player("session_id", "name"), Player("session_id", "name2"), True),
    (Player("session_id", "name"), Player("session_id2", "name"), False),
    (Player("session_id2", "name2"), Player("session_id", "name"), False)
])
def test_player_equality(player1, player2, expected):
    assert expected == (player1 == player2)
