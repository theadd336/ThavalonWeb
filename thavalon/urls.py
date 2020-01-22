from django.urls import path
from . import views

urlpatterns = [
    path("", views.HomeView.load, name="shitty_paul_index"),
    path("ViewLobbies.html", views.GameLobbiesView.load_lobbies, name="view_game_lobbies"),
    path("zzzDoNotCallcreate_new_game/", views.HomeView.create_new_game, name="create_new_game"),
    path("Lobby1/<str:room_name>/", views.room, name="room"),
    path("<str:game_id>/", views.NewLobbyView.new_game, name="new_game"),
    path("<str:room_name>/", views.room, name="room"),
]
