from channels.generic.websocket import WebsocketConsumer
from asgiref.sync import async_to_sync
import json
from game.gamemanager import GameManager
from .CommObjects import responses
from game.game_constants import MissionCard
from enum import Enum

_GAME_MANAGER = GameManager()

class IncomingMessageTypes(Enum):
    RoleInformation = 0
    SubmitVote = 1
    AllMissionInfoRequest = 2
    SubmitProposal = 3
    MoveToVote = 4
    SubmitAssassination = 5
    PlayerOrder = 6

class ChatConsumer(WebsocketConsumer):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.room_name = ""
        self.room_group_name = ""

    def connect(self):
        self.room_name = self.scope['url_route']['kwargs']['room_name']
        self.room_group_name = 'chat_%s' % self.room_name

        # Join room group
        async_to_sync(self.channel_layer.group_add)(
            self.room_group_name,
            self.channel_name
        )

        self.accept()

    def disconnect(self, close_code):
        # Leave room group
        async_to_sync(self.channel_layer.group_discard)(
            self.room_group_name,
            self.channel_name
        )

    # Receive message from WebSocket
    def receive(self, text_data):
        text_data_json = json.loads(text_data)
        message = text_data_json['message']

        # Send message to room group
        async_to_sync(self.channel_layer.group_send)(
            self.room_group_name,
            {
                'type': 'chat_message',
                'message': message
            }
        )

    # Receive message from room group
    def chat_message(self, event):
        message = event['message']

        # Send message to WebSocket
        self.send(text_data=json.dumps({
            'message': message
        }))


class LobbyConsumer(WebsocketConsumer):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.lobby_group_name = ""
        self.game_id = ""
        self.player_id = ""
        self.game = None
        self._message_types = {
            "on_player_join": self.join_game,
            "on_player_leave": self.leave_game,
            "start_game": self.start_game
        }

    def connect(self):
        self.game_id = self.lobby_group_name = self.scope["url_route"]["kwargs"]["game_id"]
        try:
            self.game = _GAME_MANAGER.get_game(self.game_id)
            self.player_id = self.scope["session"]["player_id"]
        except ValueError:
            return

        if not self.game.is_player_in_game(self.player_id):
            return
        async_to_sync(self.channel_layer.group_add)(self.lobby_group_name, self.channel_name)
        self.accept()
        self.join_game({"player_names": self.game.get_player_names_in_game()})
        return

    def disconnect(self, code):
        if code != 4000:
            self.game.remove_player(self.player_id)
        async_to_sync(self.channel_layer.group_discard)(
            self.lobby_group_name,
            self.channel_name
        )

    def receive(self, text_data):
        text_data = json.loads(text_data)
        message_type = text_data["type"]
        function_to_call = self._message_types.get(message_type)
        if function_to_call is None:
            raise NotImplementedError
        function_to_call(text_data)

    def join_game(self, joining_player_data):
        joining_player_data["type"] = "on_player_join_leave"
        async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, joining_player_data)
        return

    def on_player_join_leave(self, event):
        # Pass new player information to the listening clients
        self.send(text_data=json.dumps(event))

    def leave_game(self, _):
        response = responses.JoinLeaveGameResponse("on_player_leave")
        # Try removing the player from the game
        player_name = self.game.session_id_to_player(self.player_id)
        try:
            player_number = self.game.remove_player(self.player_id)
        except ValueError as error:
            response.error_message = "An error occurred while leaving the game: " + str(error)
            self.send(text_data=response.serialize())
            return
        # Player has been removed. Now, tell everyone listening to remove the player from the table
        response.success = True
        response.player_number = player_number
        response.player_name = player_name
        async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, response.serialize())
        return

    def on_player_leave(self, event):
        # Pass leaving player information to the listening clients
        self.send(text_data=json.dumps(event))

    def start_game(self, _):
        try:
            self.game.start_game()
        except ValueError:
            return
        async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, {"type": "on_start_game"})
        return

    def on_start_game(self, event):
        self.scope["session"]["game_id"] = self.game_id
        self.scope["session"].save()
        self.send(text_data=json.dumps(event))


class GameConsumer(WebsocketConsumer):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.lobby_group_name = ""
        self.game_id = ""
        self.player_id = ""
        self.game = None
        self._message_types = {
            IncomingMessageTypes.RoleInformation.value: self.role_information,
            IncomingMessageTypes.SubmitVote.value: self.no_op,
            IncomingMessageTypes.AllMissionInfoRequest.value: self.no_op,
            IncomingMessageTypes.SubmitProposal.value: self.no_op,
            IncomingMessageTypes.MoveToVote.value: self.no_op,
            IncomingMessageTypes.SubmitAssassination.value: self.no_op,
            IncomingMessageTypes.PlayerOrder.value: self.send_player_order
        }

    def connect(self):
        self.game_id = self.lobby_group_name = self.scope["url_route"]["kwargs"]["game_id"]
        try:
            self.game = _GAME_MANAGER.get_game(self.game_id)
            self.player_id = self.scope["session"]["player_id"]
        except ValueError:
            return

        self.accept()
        if not self.game.is_player_in_game(self.player_id):
            return
        async_to_sync(self.channel_layer.group_add)(self.lobby_group_name, self.channel_name)
        return

    def disconnect(self, code):
        async_to_sync(self.channel_layer.group_discard)(
            self.lobby_group_name,
            self.channel_name)

    def receive(self, text_data):
        text_data = json.loads(text_data)
        message_type = text_data["type"]
        function_to_call = self._message_types.get(message_type)
        if function_to_call is None:
            raise NotImplementedError
        function_to_call(text_data)

    def all_mission_info(self, _):
        pass
    
    def role_information(self, _):
        success = True
        error_message = ""
        try:
            player_info = self.game.get_player_info(self.player_id)
        except ValueError as e:
            success = False
            error_message = str(e)
            player_info = None
        response = responses.RoleInformationResponse(success, error_message, player_info)
        self.send(response.serialize())

    def send_player_order(self, _):
        success = True
        proposal_info = dict()
        try:
            proposal_info = self.game.get_proposal_info()
        except ValueError as e:
            success = False
            error_message = str(e)
        
        response = responses.PlayerOrderResponse(success, error_message, proposal_info.get("proposal_order"))
        self.send(response.serialize())
        
    def send_vote_results(self, proposal_info):
        num_upvotes = proposal_info.get("num_upvotes")
        num_downvotes = proposal_info.get("num_downvotes")

        if (num_upvotes is None or num_downvotes is None):
            raise ValueError("Votes cannot be None.")
        
        vote_info = proposal_info.get("proposal_vote_info")
        was_maeved = proposal_info.get("vote_maeved")
        last_proposal_number = self.game.current_proposal_num - 1
        if (last_proposal_number < 0):
            last_proposal_number = 0
        
        vote_info["Number of Upvotes"] = num_upvotes
        vote_info["Number of Downvotes"] = num_downvotes
        mission_number = self.game.mission_num
        response = responses.VoteResultMessage(mission_number, last_proposal_number, was_maeved, vote_info)
        self.send(response.serialize())
        
    def no_op(self, _):
        pass

    # def on_connect(self, _):
    #     response = responses.GameStateResponse()
    #     # # Try loading gamestate information. Handle any errors and tell the client about them.
    #     # try:
    #     #     gamestate = self.game.get_gamestate(self.player_id)
    #     # except ValueError as e:
    #     #     response.error_message = "Error while loading information: " + str(e)
    #     #     self.send(response.send)
    #     #     return
    #     #
    #     # # No errors. Now, we need to parse the information and send it the client
    #     # response.success = True
    #     # response.role_information = gamestate.get("role_information")
    #     # response.proposal_order = gamestate.get("proposal_order")
    #     # response.mission_sizes = gamestate.get("mission_sizes")
    #     # response.mission_results = gamestate.get("mission_results")
    #     # response.current_phase = gamestate.get("current_phase").value
    #     # response.mission_players = gamestate.get("mission_players")
    #     # response.proposer_index = gamestate.get("proposer")
    #     # response.proposal_num = gamestate.get("current_proposal_num")
    #     # response.max_num_proposals = gamestate.get("max_proposals")
    #     # response.mission_num = gamestate.get("mission_num")
    #     # response.current_proposal = gamestate.get("current_proposal")
    #     # if self.player_id == gamestate.get("proposer_id"):
    #     #     response.is_proposing = True
    #     # self.send(json.dumps(response.send()))
    #     try:
    #         player_info = self.game.get_player_info(self.player_id)
    #         proposal_info = self.game.get_proposal_info()
    #     except ValueError as e:
    #         response.error_message = "Error while loading information: " + str(e)
    #         self.send(json.dumps(response.send()))
    #         return
    #     response.success = True
    #     response.role_information = player_info
    #     response.proposal_order = proposal_info.get("proposal_order")
    #     response.proposer_index = proposal_info.get("proposer_index")
    #     response.proposal_size = proposal_info.get("proposal_size")
    #     response.max_num_proposals = proposal_info.get("max_num_proposers")
    #     response.proposal_num = proposal_info.get("current_proposal_num")
    #     response.mission_info = self.game.get_all_mission_results()
    #     response.current_phase = self.game.game_phase.value
    #     if response.current_phase == 1:
    #         response.current_proposal = self.game.current_proposals[0]
    #     if self.player_id == proposal_info.get("proposer_id"):
    #         response.is_proposing = True
    #     self.send(json.dumps(response.send()))
    #     if response.current_phase == 1:
    #         vote_response = responses.OnVoteStartResponse()
    #         vote_response.player_list = self.game.current_proposals[0]
    #         self.send(json.dumps(response.send()))
    #     if response.current_phase == 2:
    #         mission_info = self.game.get_mission_info()
    #         event = {"type": "on_mission_start", "mission_info": mission_info}
    #         self.on_mission_start(event)
    #     return

    # def propose(self, message_data):
    #     proposed_player_list = message_data["proposed_player_list"]
    #     if len(proposed_player_list) != (self.game.get_proposal_info()).get("proposal_size"):
    #         print("wrong proposal size")
    #         return
    #     response = responses.OnProposeResponse(proposed_player_list=proposed_player_list)
    #     response.proposer_name = self.game.get_player(self.game.proposer_id).name
    #     response.is_proposing = self.game.proposer_id == self.player_id
    #     async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, response.send())

    # def on_propose(self, event):
    #     # if self.player_id == self.game.proposer_id:
    #     #     return
    #     self.send(json.dumps(event))
    #     return

    # def move_to_vote(self, message):
    #     proposed_player_list = message.get("proposed_player_list")
    #     if not isinstance(proposed_player_list, list):
    #         print("invalid player list received")
    #         return
    #     if len(proposed_player_list) != self.game.get_proposal_size():
    #         print("invalid number of names received")
    #         return
    #     if self.player_id != self.game.proposer_id:
    #         print("You're not proposing!")
    #     try:
    #         new_game_state = self.game.set_proposal(proposed_player_list)
    #     except ValueError as e:
    #         print(str(e))
    #         return
    #     new_game_phase = new_game_state.get("game_phase").value
    #     del new_game_state["game_phase"]
    #     if new_game_phase == 0:
    #         new_game_state["type"] = "new_proposal"
    #         async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, new_game_state)
    #     elif new_game_phase == 1:
    #         response = responses.OnVoteStartResponse(proposed_player_list)
    #         async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, response.send())
    #     elif new_game_phase == 2:
    #         new_game_state["type"] = "on_mission_start"
    #         async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, new_game_state)
    #     else:
    #         print("Invalid gamestate at this time")
    #         return
    #     return

    # def new_proposal(self, event):
    #     proposal_info = event.get("proposal_info")
    #     response = responses.NewProposalResponse()
    #     response.is_proposing = self.player_id == proposal_info.get("proposer_id")
    #     response.proposal_order = proposal_info.get("proposal_order")
    #     response.proposer_index = proposal_info.get("proposer_index")
    #     response.proposal_size = proposal_info.get("proposal_size")
    #     response.max_num_proposals = proposal_info.get("max_num_proposers")
    #     response.proposal_num = proposal_info.get("current_proposal_num")
    #     response.proposal_vote_info = event.get("proposal_vote_info")
    #     self.send(json.dumps(response.send()))

    # def on_vote_start(self, event):
    #     self.send(json.dumps(event))

    # def vote(self, message):
    #     how_voting = message.get("how_voting")
    #     if not isinstance(how_voting, bool):
    #         print("Non-valid boolean received for voting.")
    #         return
    #     try:
    #         vote_results = self.game.set_vote(self.player_id, how_voting)
    #     except ValueError:
    #         print("already voted")
    #         return
    #     game_phase = vote_results.get("game_phase").value
    #     del vote_results["game_phase"]
    #     if game_phase == 1:
    #         self._votes_still_in_progress(vote_results.get("vote"))
    #     elif game_phase == 2:
    #         vote_results["type"] = "on_mission_start"
    #         async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, vote_results)
    #     elif game_phase == 0:
    #         vote_results["type"] = "new_proposal"
    #         async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, vote_results)
    #     else:
    #         print("invalid gamestate for voting phase")
    #     return

    # def _votes_still_in_progress(self, submitted_vote):
    #     response = responses.OnVoteResultsResponse(message_type="vote_still_in_progress")
    #     response.submitted_vote = submitted_vote
    #     self.send(json.dumps(response.send()))

    # def on_mission_start(self, event):
    #     mission_info = event.get("mission_info")
    #     response = responses.OnVoteResultsResponse(message_type="on_mission_start")
    #     response.is_on_mission = self.player_id in mission_info.get("mission_session_ids")
    #     response.player_list = mission_info.get("mission_players")
    #     response.proposal_vote_info = event.get("proposal_vote_info")
    #     self.send(json.dumps(response.send()))

    # def play_card(self, message):
    #     card = message.get("card")
    #     if card not in range(0, 3):
    #         print("Card integer not valid. Played %d." % card)
    #         return
    #     if card == MissionCard.SUCCESS.value:
    #         mission_card = MissionCard.SUCCESS
    #     elif card == MissionCard.FAIL.value:
    #         mission_card = MissionCard.FAIL
    #     elif card == MissionCard.REVERSE.value:
    #         mission_card = MissionCard.REVERSE
    #     else:
    #         print("Non-valid card to enum conversion")
    #         return
    #     try:
    #         mission_results = self.game.play_mission_card(self.player_id, mission_card)
    #     except ValueError as e:
    #         print(str(e))
    #         return
    #     game_phase = mission_results.get("game_phase").value
    #     mission_result = mission_results.get("mission_result")
    #     if mission_result is not None:
    #         mission_result = mission_result.value

    #     event = {
    #         "type": "on_mission_results",
    #         "game_phase": game_phase,
    #         "mission_result": mission_result,
    #         "card_played": mission_results.get("card_played"),
    #         "proposal_info": mission_results.get("proposal_info"),
    #         "played_cards": mission_results.get("played_cards"),
    #         "mission_players": mission_results.get("mission_players")
    #         }

    #     if game_phase == 0:
    #         async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, event)
    #     elif mission_results.get("game_phase").value == 2:
    #         self._mission_still_in_progress(mission_card.name)
    #     else:
    #         async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, event)
    #     return

    # def _mission_still_in_progress(self, card_played):
    #     response = responses.OnMissionResultsResponse("mission_still_in_progress")
    #     response.card_played = card_played
    #     self.send(json.dumps(response.send()))

    # def on_mission_results(self, event):
    #     game_phase = event.get("game_phase")
    #     response = responses.OnMissionResultsResponse(message_type="on_mission_results")
    #     response.mission_result = event.get("mission_result")
    #     response.prior_mission_num = self.game.mission_num
    #     response.played_cards = event.get("played_cards")
    #     response.players_on_mission = event.get("mission_players")
    #     self.send(json.dumps(response.send()))
    #     if game_phase == 0:
    #         self.new_proposal(event)
    #     else:
    #         print("Game Over")

    def use_ability(self):
        pass
