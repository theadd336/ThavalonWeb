import pytest
from game.playermanager import PlayerManager


def test_add_player():
    pm = PlayerManager()
    pm.add_player("session_id", "name")
    assert "session_id" in pm.session_id_to_player
    assert pm.session_id_to_player["session_id"].name == "name"


def test_add_player_existing_session_id_errors():
    pm = PlayerManager()
    pm.add_player("session_id", "name")
    with pytest.raises(ValueError):
        pm.add_player("session_id", "name2")


def test_add_player_existing_name_errors():
    pm = PlayerManager()
    pm.add_player("session _id", "name")
    with pytest.raises(ValueError):
        pm.add_player("session_id2", "name")


def test_get_player():
    pm = PlayerManager()
    pm.add_player("session_id", "name")
    assert pm.get_player("session_id").name == "name"


def test_get_nonexistent_player_errors():
    pm = PlayerManager()
    with pytest.raises(ValueError):
        pm.get_player("session_id")


def test_remove_player():
    pm = PlayerManager()
    pm.add_player("session_id", "name")
    pm.remove_player("session_id")


def test_remove_nonexistent_player_errors():
    pm = PlayerManager()
    with pytest.raises(ValueError):
        pm.remove_player("session_id")