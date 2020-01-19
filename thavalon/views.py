from django.shortcuts import render
from django.http import HttpResponse
from django.views.generic import View, TemplateView
from game.game import ThavalonGame
# Create your views here.


class HomeView(TemplateView):
    template_name = "thavalon/index.html"

    # def index(self, request):
    #     return render(request, "thavalon/index.html", context)
    #
    # def spectate_game(self, request, game_id):
    #     response = "You're currently spectating game %s."
    #     return HttpResponse(response % request.session["current_game"])
    #
    # def do_not_open(self, request, game_id):
    #     response = "You're viewing the DoNotOpen for game %s."
    #     return HttpResponse(response % game_id)
    #
    # def new_game(self, request):
    #     request.session.flush()
    #     game_id = GameManager.create_new_game()
    #     request.session["current_game"] = game_id
    #     return render(request, "thavalon/cookiejar.html", {})


class NewGameView(TemplateView):
    template_name = "thavalon/cookiejar.html"

    @staticmethod
    def new_game(request):
        request.session.flush()
        request.session["current_game"] = "testing"
        return render(request, "thavalon/cookiejar.html", {})