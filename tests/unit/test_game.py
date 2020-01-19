import pytest
from game.game import Game
from unittest.mock import Mock


@pytest.mark.parametrize("session_ids, player_names, expected_num_players", [
    ([], [], 0),
    (["0"], ["TEST"], 1),
    (["1", "2"], ["A", "B"], 2)
])
def test_get_num_players(session_ids, player_names, expected_num_players) -> None:
    game = Game()
    for session_id, player_name in zip(session_ids, player_names):
        game.add_player(session_id, player_name)
    assert game.get_num_players() == expected_num_players


@pytest.mark.parametrize("num_game_players, expected_full_game", [
    (0, False),
    (1, False),
    (9, False),
    (10, True),
    (11, False)
])
def test_is_game_full(num_game_players, expected_full_game) -> None:
    game = Game()
    mock_get_num_players = Mock()
    mock_get_num_players.return_value = num_game_players
    game.get_num_players = mock_get_num_players
    assert game.is_game_full() == expected_full_game


def test_add_existing_player_errors() -> None:
    game = Game()
    game.add_player("session_id", "name")
    with pytest.raises(ValueError):
        game.add_player("session_id", "new name")


def test_add_player() -> None:
    game = Game()
    game.add_player("session_id", "name")
    assert "session_id" in game.player_manager.session_id_to_player
    assert game.player_manager.session_id_to_player["session_id"].name == "name"


def test_remove_player() -> None:
    game = Game()
    game.add_player("session_id", "name")
    game.remove_player("session_id")
    assert "session_id" not in game.player_manager.session_id_to_player