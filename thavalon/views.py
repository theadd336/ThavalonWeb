from django.shortcuts import render
from django.http import HttpResponse
from django.template import loader
from game.game import ThavalonGame
# Create your views here.


class HomeView:
    @staticmethod
    def index(request):
        current_game = ThavalonGame(5)
        context = {"current_game": current_game}
        return render(request, "thavalon/index.html", context)

    @staticmethod
    def spectate_game(request, game_id):
        response = "You're currently spectating game %s."
        return HttpResponse(response % game_id)

    @staticmethod
    def do_not_open(request, game_id):
        response = "You're viewing the DoNotOpen for game %s."
        return HttpResponse(response % game_id)
