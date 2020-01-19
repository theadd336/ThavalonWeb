<<<<<<< HEAD
from django.urls import path
from .views import HomeView, NewGameView

urlpatterns = [
    path("", HomeView.as_view(), name="index"),
    path("newgame/", NewGameView.new_game, name="newGame"),
    path("<int:game_id>/", HomeView.spectate_game, name="spectate"),
    # path("<int:game_id>/donotopen", HomeView.do_not_open, name="donotopen")
]
=======
from django.urls import path
from .views import HomeView, NewLobbyView

urlpatterns = [
    path("", HomeView.as_view(), name="index"),
    path("lobby/", NewLobbyView.new_game, name="new_game"),
    path("joingame", NewLobbyView.join_game, name="joingame"),
    path("<int:game_id>/", HomeView.spectate_game, name="spectate"),
    # path("<int:game_id>/donotopen", HomeView.do_not_open, name="donotopen")
]
>>>>>>> 846c3935a93ed2d0b5e183fdf9fb26dee1c7c5eb
