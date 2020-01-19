from django.urls import path
from .views import HomeView, NewGameView

urlpatterns = [
    path("", HomeView.as_view(), name="index"),
    path("newgame/", NewGameView.new_game, name="newGame"),
    # path("<int:game_id>/", HomeView.spectate_game, name="spectate"),
    # path("<int:game_id>/donotopen", HomeView.do_not_open, name="donotopen")
]
