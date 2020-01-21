from channels.generic.websocket import WebsocketConsumer
from asgiref.sync import async_to_sync
import json
from game.gamemanager import GameManager
from .CommObjects import responses

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
        print(text_data_json)
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

    def connect(self):
        self.lobby_group_name = self.scope["url_route"]["kwargs"]["game_id"]
        async_to_sync(self.channel_layer.group_add)(self.lobby_group_name, self.channel_name)
        self.accept()

    def disconnect(self, code):
        async_to_sync(self.channel_layer.group_discard)(
            self.lobby_group_name,
            self.channel_name
        )

    def receive(self, text_data):
        self.join_game(text_data)

    def join_game(self, joining_player_data):
        # Load the player's name from the json object.

        player_name = json.loads(joining_player_data)["player_name"]
        # Load the session, game ID, and player ID, and game. The game should exist
        # at this point, since it's created when a new game is created.
        session = self.scope["session"]
        game_id = self.lobby_group_name
        player_id = session["player_id"]
        game = _GAME_MANAGER.get_game(game_id)
        response = responses.JoinGameResponse()
        # Try joining the game. If we encounter errors, tell only the client requesting to join the game.
        # Return the error message.
        try:
            player_number = game.add_player(player_id, player_name)
        except ValueError as e:
            response.error_message = "An error occurred while joining the game: " + str(e)
            self.send(text_data=json.dumps(response.send()))
            return
        # We successfully joined the game. Tell the caller that and send our new player's information
        # to all listeners.
        response.success = True
        response.player_number = 1
        print(player_name)
        print(player_number)
        async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, {
            "type": "on_player_join",
            "player_name": player_name,
            "player_number": 1
        })
        self.send(text_data=json.dumps(response.send()))
        return

    def on_player_join(self, event):
        # Pass new player information to the listening clients
        self.send(text_data=json.dumps({
            "type": "on_player_join",
            "new_player_name": event["player_name"],
            "new_player_number": event["player_number"]
        }))
