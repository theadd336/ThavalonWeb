import random
import pytest
from game.game import Game, GameState
from game.player import Player
from typing import List
from unittest.mock import Mock


@pytest.mark.parametrize("session_ids, player_names, expected_num_players", [
    ([], [], 0),
    (["0"], ["TEST"], 1),
    (["1", "2"], ["A", "B"], 2)
])
def test_get_num_players(session_ids: List[str], player_names: List[str], expected_num_players: int) -> None:
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
def test_is_game_full(num_game_players: int, expected_full_game: bool) -> None:
    game = Game()
    mock_get_num_players = Mock()
    mock_get_num_players.return_value = num_game_players
    game.get_num_players = mock_get_num_players
    assert game.is_game_full() == expected_full_game


def test_get_players_in_game() -> None:
    game = Game()
    game.add_player("session_id", "name")
    assert game.get_player_names_in_game() == ["name"]
    game.add_player("session_id2", "name2")
    assert game.get_player_names_in_game() == ["name", "name2"]


def test_add_player_full_game_fails() -> None:
    game = Game()
    mock_is_game_full = Mock()
    mock_is_game_full.return_value = True
    game.is_game_full = mock_is_game_full
    with pytest.raises(ValueError):
        game.add_player("session_id", "name")


def test_add_player() -> None:
    game = Game()
    assert game.add_player("session_id", "name") == ["name"]
    assert "session_id" in game.session_id_to_player
    assert game.session_id_to_player["session_id"].name == "name"
    assert game.add_player("session_id2", "name2") == ["name", "name2"]


def test_add_player_existing_session_id_errors() -> None:
    game = Game()
    game.add_player("session_id", "name")
    with pytest.raises(ValueError):
        game.add_player("session_id", "name2")


def test_add_player_existing_name_errors() -> None:
    game = Game()
    game.add_player("session _id", "name")
    with pytest.raises(ValueError):
        game.add_player("session_id2", "name")


def test_get_player() -> None:
    game = Game()
    game.add_player("session_id", "name")
    assert game.get_player("session_id").name == "name"


def test_get_nonexistent_player_errors() -> None:
    game = Game()
    with pytest.raises(ValueError):
        game.get_player("session_id")


def test_remove_player() -> None:
    game = Game()
    game.add_player("session_id", "name")
    game.add_player("session_id2", "name2")
    assert game.remove_player("session_id") == ["name2"]


def test_remove_nonexistent_player_errors() -> None:
    game = Game()
    with pytest.raises(ValueError):
        game.remove_player("session_id")


def test_is_player_in_game_false() -> None:
    game = Game()
    assert not game.is_player_in_game("FAKE")


@pytest.mark.parametrize("session_id", ["session_id", "test"])
def test_is_player_in_game_true(session_id) -> None:
    game = Game()
    game.add_player(session_id, "name")
    assert game.is_player_in_game(session_id)


def test_get_starting_info_invalid_player_errors() -> None:
    game = Game()
    with pytest.raises(ValueError):
        game.get_starting_info("session_id")


def test_get_starting_info() -> None:
    game = Game()
    mock_get_num_players = Mock()
    mock_get_num_players.return_value = 5
    game.get_num_players = mock_get_num_players
    game.proposal_order = ["1", "2", "3", "4", "5"]  # hard code for tests
    game.add_player("session_id", "name")
    result = game.get_starting_info("session_id")
    assert result["player_role"] is None  # None because will be generated in start game
    assert result["proposal_order"] == ["1", "2", "3", "4", "5"]
    assert result["first_proposer"] == "4"
    assert result["second_proposer"] == "5"
    assert result["num_on_mission"] == 2


@pytest.mark.parametrize("game_state", [GameState.IN_PROGRESS, GameState.DONE])
def test_add_player_not_in_lobby_ends(game_state):
    game = Game()
    game.game_state = game_state
    with pytest.raises(ValueError) as exc:
        game.add_player("session_id", "name")
    assert str(exc.value) == "Can only add player while in lobby."


@pytest.mark.parametrize("game_state", [GameState.IN_PROGRESS, GameState.DONE])
def test_remove_player_not_in_lobby_ends(game_state):
    game = Game()
    game.add_player("session_id", "name")
    game.game_state = game_state
    with pytest.raises(ValueError) as exc:
        game.remove_player("session_id")
    assert str(exc.value) == "Can only remove player while in lobby."

# TODO: Add back in this test
"""
@pytest.mark.parametrize("number_of_players", [4, 11])
def test_start_game_invalid_number_players(number_of_players: int) -> None:
    game = Game()
    mock_get_num_players = Mock()
    mock_get_num_players.return_value = number_of_players
    game.get_num_players = mock_get_num_players

    with pytest.raises(ValueError):
        game.start_game()
"""


def test_start_game_verify_proposal_order() -> None:
    game = Game()
    game.add_player("session_id1", "name1")
    game.add_player("session_id2", "name2")
    game.add_player("session_id3", "name3")
    game.add_player("session_id4", "name4")
    game.add_player("session_id5", "name5")

    random.seed(0)  # set seed to 0 so proposal order will be consistent
    game.start_game()
    assert game.proposal_order == ["name3", "name1", "name2", "name5", "name4"]
    # TODO: Test roles assign properly


@pytest.mark.parametrize("num_players, session_id_to_player", [
    (
        2,
        {
            "id1": Player("id1", "Tyler"),
            "id2": Player("id2", "Jesse")
        }
    ),
    (
        3,
        {
            "id1": Player("id1", "Galen"),
            "id2": Player("id2", "Colin"),
            "id3": Player("id3", "Darcy")
        }
    ),
    (
        5,
        {
            "id1": Player("id1", "Andrew"),
            "id2": Player("id2", "Arya"),
            "id3": Player("id3", "Jared"),
            "id4": Player("id4", "Meg"),
            "id5": Player("id5", "Paul")
        }
    )
])
def test_start_game_players_assigned(num_players, session_id_to_player) -> None:
    game = Game()
    assert game.game_state == GameState.IN_LOBBY
    mock_get_num_players = Mock()
    mock_get_num_players.return_value = num_players
    game.get_num_players = mock_get_num_players

    game.session_id_to_player = session_id_to_player
    game.start_game()

    seen_roles = []
    for player in session_id_to_player.values():
        assert player.role not in seen_roles
        seen_roles.append(player.role)

    assert game.game_state == GameState.IN_PROGRESS
