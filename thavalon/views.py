from django.shortcuts import render
from django.http import HttpResponse, JsonResponse
from django.views.generic import View, TemplateView
from game.gamemanager import GameManager
from .lobbymanager import LobbyManager
import uuid
# Create your views here.

_GAME_MANAGER = GameManager()
_LOBBY_MANAGER = LobbyManager()

class HomeView(View):
    @staticmethod
    def load(request):
        template_name = "thavalon/index.html"
        return render(request, template_name)

    @staticmethod
    def create_new_game(request):
        request.session.flush()
        request.session["game_id"] = _GAME_MANAGER.create_new_game()
        request.session["player_id"] = str(uuid.uuid4())
        response = {"game_id": request.session["game_id"]}
        return JsonResponse(response)

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
    template_name = "thavalon/LobbyWaiting.html"

    @staticmethod
    def new_game(request, game_id):
        return render(request, "thavalon/LobbyWaiting.html", {"game_id": game_id})

    @staticmethod
    def join_game(request):
        player_id = request.session.get("player_id")
        game_id = request.session.get("game_id")
        print(game_id)
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


class GameLobbiesView():
    template_name = "thavalon/ViewLobbies.html"

    @staticmethod
    def load_lobbies(request):
        return render(request, GameLobbiesView.template_name)

def room(request, room_name):
    return render(request, "thavalon/room.html", {'room_name': room_name})
