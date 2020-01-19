import pytest
from game.game import Game
from game.gamemanager import GameManager


def test_create_new_game() -> None:
    gm = GameManager()
    assert isinstance(gm.create_new_game(), str)


def test_get_game() -> None:
    gm = GameManager()
    uuid = gm.create_new_game()
    assert isinstance(gm.get_game(uuid), Game)


def test_get_invalid_game_errors() -> None:
    gm = GameManager()
    with pytest.raises(ValueError):
        gm.get_game("FAKE_UUID")


def test_delete_game() -> None:
    gm = GameManager()
    uuid = gm.create_new_game()
    gm.delete_game(uuid)
    with pytest.raises(ValueError):
        gm.get_game(uuid)
