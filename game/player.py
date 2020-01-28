class Player:
    def __init__(self, session_id: str, name: str) -> None:
        self.session_id = session_id
        self.name = name
        self.role: 'Role' = None  # noqa
