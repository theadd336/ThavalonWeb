from channels.generic.websocket import WebsocketConsumer
from asgiref.sync import async_to_sync
import json
from game.gamemanager import GameManager
from .CommObjects import responses
from enum import Enum

_GAME_MANAGER = GameManager()


class ChatConsumer(WebsocketConsumer):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.room_name = ""
        self.room_group_name = ""

    def connect(self):
        self.room_name = self.scope['url_route']['kwargs']['room_name']
        self.room_group_name = 'chat_%s' % self.room_name

        # Join room group
        async_to_sync(self.channel_layer.group_add)(
            self.room_group_name,
            self.channel_name
        )

        self.accept()

    def disconnect(self, close_code):
        # Leave room group
        async_to_sync(self.channel_layer.group_discard)(
            self.room_group_name,
            self.channel_name
        )

    # Receive message from WebSocket
    def receive(self, text_data):
        text_data_json = json.loads(text_data)
        message = text_data_json['message']

        # Send message to room group
        async_to_sync(self.channel_layer.group_send)(
            self.room_group_name,
            {
                'type': 'chat_message',
                'message': message
            }
        )

    # Receive message from room group
    def chat_message(self, event):
        message = event['message']

        # Send message to WebSocket
        self.send(text_data=json.dumps({
            'message': message
        }))


class LobbyConsumer(WebsocketConsumer):

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.lobby_group_name = ""
        self.game_id = ""
        self.player_id = ""
        self.game = None
        self._message_types = {
            "on_player_join": self.join_game,
            "on_player_leave": self.leave_game,
            "start_game": self.start_game
        }

    def connect(self):
        self.game_id = self.lobby_group_name = self.scope["url_route"]["kwargs"]["game_id"]
        self.game = _GAME_MANAGER.get_game(self.game_id)
        self.player_id = self.scope["session"]["player_id"]
        async_to_sync(self.channel_layer.group_add)(self.lobby_group_name, self.channel_name)
        self.accept()

    def disconnect(self, code):
        self.game.remove_player(self.player_id)
        self.scope["session"].flush()
        self.scope["session"].save()
        async_to_sync(self.channel_layer.group_discard)(
            self.lobby_group_name,
            self.channel_name
        )

    def receive(self, text_data):
        text_data = json.loads(text_data)
        message_type = text_data["type"]
        function_to_call = self._message_types.get(message_type)
        if function_to_call is None:
            raise NotImplementedError
        function_to_call(text_data)

    def join_game(self, joining_player_data):
        # Load the player's name from the json object.
        player_name = joining_player_data["player_name"]
        # Load the session, game ID, and player ID, and game. The game should exist
        # at this point, since it's created when a new game is created.
        game = _GAME_MANAGER.get_game(self.game_id)
        response = responses.JoinLeaveGameResponse("on_player_join")
        # Try joining the game. If we encounter errors, tell only the client requesting to join the game.
        # Return the error message.
        try:
            player_number = self.game.add_player(self.player_id, player_name)
        except ValueError as e:
            response.error_message = "An error occurred while joining the game: " + str(e)
            self.send(text_data=json.dumps(response.send()))
            return
        # We successfully joined the game. Tell the caller that and send our new player's information
        # to all listeners.
        response.player_number = player_number
        async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, response.send())
        return

    def on_player_join(self, event):
        # Pass new player information to the listening clients
        self.send(text_data=json.dumps(event))

    def leave_game(self, _):
        response = responses.JoinLeaveGameResponse("on_player_leave")
        # Try removing the player from the game
        player_name = self.game.session_id_to_player(self.player_id)
        try:
            player_number = self.game.remove_player(self.player_id)
        except ValueError as error:
            response.error_message = "An error occurred while leaving the game: " + str(error)
            self.send(text_data=json.dumps(response.send()))
            return
        # Player has been removed. Now, tell everyone listening to remove the player from the table
        response.success = True
        response.player_number = player_number
        response.player_name = player_name
        async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, response.send())
        return

    def on_player_leave(self, event):
        # Pass leaving player information to the listening clients
        self.send(text_data=json.dumps(event))

    def start_game(self, _):
        try:
            self.game.start_game()
        except ValueError:
            return
        async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, {"type": "on_start_game"})
        return

    def on_game_start(self, event):
        self.scope["session"]["game_id"] = self.game_id
        self.scope["session"].save()
        self.send(json.dumps(event))
