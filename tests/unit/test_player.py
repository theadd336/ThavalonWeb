import pytest
from game.player import Player


@pytest.mark.parametrize("player1, player2, expected", [
    (Player("session_id", "name"), Player("session_id", "name"), True),
    (Player("session_id", "name"), Player("session_id", "name2"), True),
    (Player("session_id", "name"), Player("session_id2", "name"), False),
    (Player("session_id2", "name2"), Player("session_id", "name"), False)
])
def test_player_equality(player1, player2, expected) -> None:
    assert expected == (player1 == player2)

@pytest.mark.parametrize("player, expected", [
    (Player("session_id", "name"), "Player: name"),
    (Player("session_id2", "name2"), "Player: name2")
])
def test_player_repr(player, expected) -> None:
    assert repr(player) == expected
