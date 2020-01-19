from game.singleton import Singleton


def test_singleton_create_twice_returns_same_obj() -> None:
    class Test(metaclass=Singleton):
        pass

    test1 = Test()
    test2 = Test()
    assert test1 is test2
