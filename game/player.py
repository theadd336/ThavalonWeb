class Player:
    def __init__(self, session_id: str, name: str) -> None:
        self.session_id = session_id
        self.name = name
        self.role: 'Role' = None  # noqa

    def __eq__(self, other) -> bool:
        if not isinstance(other, self.__class__):
            return False
        return self.session_id == other.session_id

    def __ne__(self, other) -> bool:
        return not self.__eq__(other)

    def __repr__(self) -> str:
        return f"Player: {self.name}"
