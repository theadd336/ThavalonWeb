from django.shortcuts import render
from django.http import HttpResponse, JsonResponse
from django.views.generic import View, TemplateView
from game.gamemanager import GameManager
import uuid
import channels
# Create your views here.


class HomeView(TemplateView):
    template_name = "thavalon/index.html"

    # def index(self, request):
    #     return render(request, "thavalon/index.html", context)
    #
    @staticmethod
    def spectate_game(request, game_id):
        response = "You're currently spectating game %s. The number of players is %d."
        game_manager = GameManager()
        game_id = request.session["current_game"]
        game = game_manager.get_game(game_id)
        return HttpResponse(response % (request.session["current_game"], game.num_players))
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


class NewLobbyView(View):
    template_name = "thavalon/lobby.html"
    @staticmethod
    def new_game(request):
        request.session.flush()
        game_manager = GameManager()
        request.session["current_game"] = game_manager.create_new_game()
        request.session["player_id"] = str(uuid.uuid4())
        return render(request, "thavalon/lobby.html", {})

    @staticmethod
    def join_game(request):
        player_id = request.session.get("player_id")
        game_id = request.session.get("current_game")
        game_manager = GameManager()
        current_game = game_manager.get_game(game_id)
        response = {"success": 0}
        if current_game is None:
            response["error"] = "This game does not exist."
            return JsonResponse(response)
        try:
            player_number = current_game.add_player(player_id, "Paul!")
        except ValueError as error:
            response["error"] = "An error occurred while joining the game: " + str(error)
            return JsonResponse(response)
        response["success"] = 1
        response["number"] = 1
        response["name"] = "Paul"
        return JsonResponse(response)


class GameLobbiesView(TemplateView):
    template_name = "thavalon/gamelobbies.html"


def room(request, room_name):
    return render(request, 'chat/room.html', {'room_name': room_name})
