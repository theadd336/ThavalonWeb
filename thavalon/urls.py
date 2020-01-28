from django.urls import path
from . import views

urlpatterns = [
    path("", views.HomeView.load, name="shitty_paul_index"),
    path("ViewLobbies.html", views.GameLobbiesView.load_lobbies, name="view_game_lobbies"),
    path("zzzDoNotCallcreate_new_game/", views.GameLobbiesView.create_new_game, name="create_new_game"),
    path("zzzDoNotCallJoin_Game/", views.GameLobbiesView.join_game, name="join_game"),
    path("Lobby1/<str:room_name>/", views.room, name="room"),
    path("<str:game_id>/", views.LobbyWaitingView.load, name="join_lobby"),
    path("<str:room_name>/", views.room, name="room"),
]
