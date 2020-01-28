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


class LobbyWaitingView(View):
    template_name = "thavalon/LobbyWaiting.html"

    @staticmethod
    def load(request, game_id):
        return render(request, "thavalon/LobbyWaiting.html", {"game_id": game_id})


class GameLobbiesView:
    template_name = "thavalon/ViewLobbies.html"

    @staticmethod
    def load_lobbies(request):
        return render(request, GameLobbiesView.template_name)

    @staticmethod
    def create_new_game(request):
        request.session.flush()
        game_id = _GAME_MANAGER.create_new_game()
        lobby_id = _LOBBY_MANAGER.create_new_lobby(game_id)
        request.session["game_id"] = game_id
        request.session["player_id"] = str(uuid.uuid4())
        response = {"lobby_id": lobby_id}
        print(game_id)
        return JsonResponse(response)

    @staticmethod
    def join_game(request):
        player_id = str(uuid.uuid4())
        player_name = str(request.POST.get("player_name"))
        lobby_id = str(request.POST.get("lobby_id"))
        game_id = _LOBBY_MANAGER.get_game_from_lobby(lobby_id)

        current_game = _GAME_MANAGER.get_game(game_id)
        response = {"success": 0}
        if current_game is None:
            response["error"] = "This game does not exist."
            return JsonResponse(response)
        try:
            player_list = current_game.add_player(player_id, player_name)
        except ValueError as error:
            response["error"] = "An error occurred while joining the game: " + str(error)
            return JsonResponse(response)
        response["success"] = 1
        response["game_id"] = game_id
        return JsonResponse(response)

def room(request, room_name):
    return render(request, "thavalon/room.html", {'room_name': room_name})
