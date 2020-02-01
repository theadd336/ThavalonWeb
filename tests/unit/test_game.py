import random
import pytest
from game.game import Game, GamePhase, LobbyStatus
from game.player import Player
from game.role import Team
from typing import List
from unittest.mock import Mock, PropertyMock


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


@pytest.mark.parametrize("lobby_status", [LobbyStatus.IN_PROGRESS, LobbyStatus.DONE])
def test_add_player_not_in_lobby_ends(lobby_status):
    game = Game()
    game.lobby_status = lobby_status
    with pytest.raises(ValueError) as exc:
        game.add_player("session_id", "name")
    assert str(exc.value) == "Can only add player while in lobby."


@pytest.mark.parametrize("lobby_status", [LobbyStatus.IN_PROGRESS, LobbyStatus.DONE])
def test_remove_player_not_in_lobby_ends(lobby_status):
    game = Game()
    game.add_player("session_id", "name")
    game.lobby_status = lobby_status
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
    p1 = Player("session_id1", "name1")
    p2 = Player("session_id2", "name2")
    p3 = Player("session_id3", "name3")
    p4 = Player("session_id4", "name4")
    p5 = Player("session_id5", "name5")
    game.add_player("session_id1", "name1")
    game.add_player("session_id2", "name2")
    game.add_player("session_id3", "name3")
    game.add_player("session_id4", "name4")
    game.add_player("session_id5", "name5")

    random.seed(0)  # set seed to 0 so proposal order will be consistent
    game.start_game()
    assert game.proposal_order_names == ["name3", "name1", "name2", "name5", "name4"]
    assert game.proposal_order_players == [p3, p1, p2, p5, p4]


@pytest.mark.repeat(10)
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
    assert game.lobby_status == LobbyStatus.JOINING
    mock_get_num_players = Mock()
    mock_get_num_players.return_value = num_players
    game.get_num_players = mock_get_num_players

    game.session_id_to_player = session_id_to_player
    game.start_game()

    seen_role_names = []
    for player in session_id_to_player.values():
        assert player.role.role_name not in seen_role_names
        seen_role_names.append(player.role.role_name)

    if "Tristan" in seen_role_names:
        assert "Iseult" in seen_role_names
    if "Iseult" in seen_role_names:
        assert "Tristan" in seen_role_names

    assert game.lobby_status == LobbyStatus.IN_PROGRESS


@pytest.mark.parametrize("lobby_status", [LobbyStatus.JOINING, LobbyStatus.DONE])
def test_get_player_info_invalid_lobby(lobby_status):
    game = Game()
    game.lobby_status = lobby_status
    with pytest.raises(ValueError):
        game.get_player_info("session_id")


def test_get_player_info():
    game = Game()
    game.lobby_status = LobbyStatus.IN_PROGRESS
    mock_role = Mock()
    type(mock_role).role_name = PropertyMock(return_value="Role")
    type(mock_role).team = PropertyMock(return_value=Team.EVIL)
    mock_role.get_description.return_value = "Description"

    mock_player = Mock()
    type(mock_player).role = mock_role

    mock_get_player = Mock()
    type(mock_get_player).role = PropertyMock(return_value=mock_role)
    mock_get_player.return_value = mock_player
    game.get_player = mock_get_player
    result = game.get_player_info("test")

    assert result["role"] == "Role"
    assert result["team"] == 1
    assert result["information"] == "Description"


@pytest.mark.parametrize("lobby_status", [LobbyStatus.JOINING, LobbyStatus.DONE])
def test_get_proposal_info_invalid_lobby(lobby_status):
    game = Game()
    game.lobby_status = lobby_status
    with pytest.raises(ValueError):
        game.get_proposal_info()


def test_get_proposal_info():
    game = Game()
    game.lobby_status = LobbyStatus.IN_PROGRESS
    game.proposal_order_names = ["test", "name", "list"]
    game.proposer_id = "test"
    game.proposer_index = 0
    game.mission_num = 0
    game.max_num_proposers = 2
    game.game_phase = GamePhase.PROPOSAL

    mock_get_num_players = Mock()
    mock_get_num_players.return_value = 5
    game.get_num_players = mock_get_num_players

    result = game.get_proposal_info()
    assert result["proposal_order"] == ["test", "name", "list"]
    assert result["proposer_id"] == "test"
    assert result["proposer_index"] == 0
    assert result["proposal_size"] == 2
    assert result["max_num_proposers"] == 2
    assert result["game_phase"] == GamePhase.PROPOSAL


@pytest.mark.parametrize("lobby_status", [LobbyStatus.JOINING, LobbyStatus.DONE])
def test_get_round_info_invalid_lobby(lobby_status):
    game = Game()
    game.lobby_status = lobby_status
    with pytest.raises(ValueError):
        game.get_round_info()


@pytest.mark.parametrize("mission_num, num_players, expected_info", [
    (0, 5, "The first mission has only two proposals. No voting will happen until both proposals are " \
           "made. Upvote for the first proposal, downvote for the second proposal."),
    (1, 5, ""),
    (2, 5, ""),
    (3, 5, ""),
    (3, 7, "There are two fails required for this mission to fail."),
    (4, 7, "")
])
def test_get_round_info(mission_num, num_players, expected_info) -> None:
    game = Game()
    game.lobby_status = LobbyStatus.IN_PROGRESS
    mock_get_num_players = Mock()
    mock_get_num_players.return_value = num_players
    game.get_num_players = mock_get_num_players
    game.mission_num = mission_num

    result = game.get_round_info()
    assert result["mission_num"] == mission_num
    assert result["mission_info"] == expected_info


@pytest.mark.parametrize("lobby_status", [LobbyStatus.JOINING, LobbyStatus.DONE])
def test_set_proposal_info_invalid_lobby(lobby_status):
    game = Game()
    game.lobby_status = lobby_status
    with pytest.raises(ValueError):
        game.set_proposal([])


def test_set_proposal_round_1():
    game = Game()
    mock_get_num_players = Mock()
    mock_get_num_players.return_value = 5
    game.get_num_players = mock_get_num_players
    game.mission_num = 0
    game.proposer_index = 3
    game.proposal_order_names = ["player1", "player2", "player3", "player4", "player5"]
    game.proposal_order_players = [
        Player("1", "player1"),
        Player("2", "player2"),
        Player("3", "player3"),
        Player("4", "player4"),
        Player("5", "player5")
    ]
    game.lobby_status = LobbyStatus.IN_PROGRESS

    result1 = game.set_proposal(["player1", "player2"])
    assert result1["game_phase"] == GamePhase.PROPOSAL
    assert result1["proposals"] == [["player1", "player2"]]
    assert result1["proposal_info"] == {
        "proposal_order": ["player1", "player2", "player3", "player4", "player5"],
        "proposer_id": "5",
        "proposer_index": 4,
        "proposal_size": 2,
        "max_num_proposers": 2,
        "game_phase": GamePhase.PROPOSAL
    }

    result2 = game.set_proposal(["player3", "player1"])
    assert result2["game_phase"] == GamePhase.VOTE
    assert result2["proposals"] == [["player1", "player2"], ["player3", "player1"]]
    assert game.proposer_index == 0
    assert game.proposer_id == "1"


@pytest.mark.parametrize("mission_num, proposal, expected_result", [
    (1, ["p1", "p2", "p3"], {
        "game_phase": GamePhase.VOTE,
        "proposals": [["p1", "p2", "p3"]]
    }),
    (2, ["p3", "p5"], {
        "game_phase": GamePhase.VOTE,
        "proposals": [["p3", "p5"]]
    }),
    (3, ["p3", "p4", "p2"], {
        "game_phase": GamePhase.VOTE,
        "proposals": [["p3", "p4", "p2"]]
    }),
    (4, ["p1", "p3", "p5"], {
        "game_phase": GamePhase.VOTE,
        "proposals": [["p1", "p3", "p5"]]
    })
])
def test_set_proposal_all_other_rounds(mission_num, proposal, expected_result):
    game = Game()
    mock_get_num_players = Mock()
    mock_get_num_players.return_value = 5
    game.get_num_players = mock_get_num_players
    game.mission_num = mission_num
    game.proposer_index = 0
    game.proposal_order_names = ["p1", "p2", "p3", "p4", "p5"]
    game.proposal_order_players = [
        Player("1", "p1"),
        Player("2", "p2"),
        Player("3", "p3"),
        Player("4", "p4"),
        Player("5", "p5")
    ]
    game.number_votes = 3
    game.lobby_status = LobbyStatus.IN_PROGRESS
    assert game.set_proposal(proposal) == expected_result
    assert game.number_votes == 0


def test_set_proposal_with_force_starts_mission():
    game = Game()
    game.lobby_status = LobbyStatus.IN_PROGRESS
    mock_get_num_players = Mock()
    mock_get_num_players.return_value = 5
    game.get_num_players = mock_get_num_players
    game.mission_num = 1
    game.proposer_index = 0
    game.proposal_order_names = ["p1", "p2", "p3", "p4", "p5"]
    game.proposal_order_players = [
        Player("1", "p1"),
        Player("2", "p2"),
        Player("3", "p3"),
        Player("4", "p4"),
        Player("5", "p5")
    ]
    game.max_num_proposers = 3
    game.current_proposal_num = 3
    result = game.set_proposal(["p1", "p2", "p3"])
    assert result["game_phase"] == GamePhase.MISSION
    assert result["mission_players"] == ["p1", "p2", "p3"]
    assert game.current_proposal_num == 1


def test_set_invalid_proposal_size_errors():
    game = Game()
    mock_get_num_players = Mock()
    mock_get_num_players.return_value = 5
    game.get_num_players = mock_get_num_players
    game.mission_num = 0
    game.proposal_order_names = ["p1", "p2", "p3", "p4", "p5"]
    game.proposal_order_players = [
        Player("1", "p1"),
        Player("2", "p2"),
        Player("3", "p3"),
        Player("4", "p4"),
        Player("5", "p5")
    ]
    game.lobby_status = LobbyStatus.IN_PROGRESS
    with pytest.raises(ValueError) as excinfo:
        game.set_proposal(["p1"])

    assert str(excinfo.value) == "Expected proposal of size 2, but instead got ['p1']."


def test_set_invalid_proposal_size_errors():
    game = Game()
    mock_get_num_players = Mock()
    mock_get_num_players.return_value = 5
    game.get_num_players = mock_get_num_players
    game.mission_num = 0
    game.proposal_order_names = ["p1", "p2", "p3", "p4", "p5"]
    game.proposal_order_players = [
        Player("1", "p1"),
        Player("2", "p2"),
        Player("3", "p3"),
        Player("4", "p4"),
        Player("5", "p5")
    ]
    game.lobby_status = LobbyStatus.IN_PROGRESS
    with pytest.raises(ValueError) as excinfo:
        game.set_proposal(["p1", "FAKE"])

    assert str(excinfo.value) == "FAKE is not in the game."


@pytest.mark.parametrize("lobby_status", [LobbyStatus.JOINING, LobbyStatus.DONE])
def test_set_vote_invalid_lobby(lobby_status):
    game = Game()
    game.lobby_status = lobby_status
    with pytest.raises(ValueError):
        game.set_vote("session_id", False)


def test_voting():
    game = Game()
    mock_get_num_players = Mock()
    mock_get_num_players.return_value = 5
    game.get_num_players = mock_get_num_players
    game.mission_num = 0
    game.proposal_order_names = ["p1", "p2", "p3", "p4", "p5"]
    game.proposal_order_players = [
        Player("1", "p1"),
        Player("2", "p2"),
        Player("3", "p3"),
        Player("4", "p4"),
        Player("5", "p5")
    ]
    game.lobby_status = LobbyStatus.IN_PROGRESS
