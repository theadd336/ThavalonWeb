from django.urls import path
from . import views

urlpatterns = [
    path("", views.HomeView.as_view(), name="index"),
    path("lobby/", views.NewLobbyView.new_game, name="new_game"),
    path("joingame/", views.NewLobbyView.join_game, name="joingame"),
    path("viewgamelobbies/", views.GameLobbiesView.as_view(), name="view_game_lobbies"),
    path("<str:room_name>/", views.room, name="room"),
    path("<int:game_id>/", views.HomeView.spectate_game, name="spectate")
]
