from django.urls import path
from . import consumers

websocket_urlpatterns = [
    path("ws/thavalon/Lobby1/<slug:room_name>/", consumers.ChatConsumer),
    path("ws/thavalon/<slug:game_id>/", consumers.LobbyConsumer),
]
