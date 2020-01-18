from django.urls import path
from .views import HomeView

urlpatterns = [
    path("", HomeView.index, name="index"),
    path("<int:game_id>/", HomeView.spectate_game, name="spectate"),
    path("<int:game_id>/donotopen", HomeView.do_not_open, name="donotopen")
]
