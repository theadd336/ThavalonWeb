class Game:
    def __init__(self):
        self.num_players = 1
        self._player_dict = {}
        self._has_started = False
        self.player_dict = {"a": "Paul", "b": "Andrew"}
        self.UUID = "Testing"
    def add_player(self, session_id: str, player_name: str):
        if len(self._player_dict) >= self.num_players:
            raise ValueError("This game is currently full.")

        registered_name = self._player_dict.get(session_id)
        if registered_name is not None:
            raise KeyError("Player %s is already in the game." % registered_name)
        self._player_dict[session_id] = player_name

    def start_game(self):
        if self._has_started:
            raise EnvironmentError("Game already in progress.")
        self._has_started = True
        self._player_dict.keys()
        # Do things
