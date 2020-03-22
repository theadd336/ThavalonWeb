from channels.generic.websocket import WebsocketConsumer
from asgiref.sync import async_to_sync
import json
from game.gamemanager import GameManager
from .CommObjects import responses
from game.game_constants import MissionCard, GamePhase
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
    ProposalVoteInformationRequest = 7
    PlayCard = 8

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
            IncomingMessageTypes.SubmitVote.value: self.submit_vote,
            IncomingMessageTypes.AllMissionInfoRequest.value: self.send_all_mission_info,
            IncomingMessageTypes.SubmitProposal.value: self.broadcast_tentative_proposal,
            IncomingMessageTypes.MoveToVote.value: self.broadcast_moving_to_vote,
            IncomingMessageTypes.SubmitAssassination.value: self.no_op,
            IncomingMessageTypes.PlayerOrder.value: self.send_player_order,
            IncomingMessageTypes.ProposalVoteInformationRequest.value: self.send_proposal_vote_info,
            IncomingMessageTypes.PlayCard.value: self.play_card
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
        error_message = ""
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
        
    def send_all_mission_info(self, _):
        all_mission_results = self.game.get_all_mission_results()
        all_mission_info = self.game.get_all_mission_default_info()
        all_mission_info.update(all_mission_results)
        all_mission_info_list = list()
        for mission_num, result_dict in all_mission_info.items():
            result_dict["missionNum"] = mission_num
            all_mission_info_list.append(result_dict)
        
        response = responses.AllMissionInfoResponse(all_mission_info_list)
        self.send(response.serialize())

    def broadcast_tentative_proposal(self, proposal_info):
        tentative_proposal_event = {
            "type": "send_tentative_proposal",
            "proposal": proposal_info.get("proposal")
        }

        async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, tentative_proposal_event)
    
    def send_tentative_proposal(self, tentative_proposal_event):
        proposal = tentative_proposal_event.get("proposal")
        response = responses.TentativeProposalResponse(proposal)
        self.send(response.serialize())

    def send_proposal_vote_info(self, _):
        game_phase = self.game.game_phase
        if game_phase == GamePhase.PROPOSAL:
            self.send_new_proposal_info(None)
        elif game_phase == GamePhase.VOTE:
            proposal = self.game.current_proposals[-1]
            vote_info_event = {"proposal": proposal}
            self.send_vote_info(vote_info_event)
        elif game_phase == GamePhase.ASSASSINATION:
            pass
    
    def send_new_proposal_info(self, _):
        proposal_info = self.game.get_proposal_info()
        proposer_id = proposal_info.get("proposer_id")
        response = responses.NewProposalResponse(
            self.game.get_player(proposer_id).name,
            self.player_id == proposer_id,
            proposal_info.get("proposal_size"),
            proposal_info.get("current_proposal_num"),
            proposal_info.get("max_num_proposers")
        )
        self.send(response.serialize())

    def broadcast_moving_to_vote(self, proposal):
        proposal_list = proposal.get("proposal")
        game_info = self.game.set_proposal(proposal_list)
        game_phase = game_info.get("game_phase")
        if game_phase == GamePhase.PROPOSAL:
            async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, {"type": "send_new_proposal_info"})
        elif game_phase == GamePhase.MISSION:
            async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, {"type": "send_mission_info"})
            async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, {"type": "send_game_phase_update"})
        else:
            vote_info_event = {"type": "send_vote_info", "proposal": proposal_list}
            async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, vote_info_event)
            async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, {"type": "send_game_phase_update"})
        

    def send_vote_info(self, vote_info_event):
        proposal = vote_info_event.get("proposal")
        response = responses.MoveToVoteResponse(proposal)
        self.send(response.serialize())
    
    def submit_vote(self, vote_info):
        vote = vote_info.get("vote")
        game_info = self.game.set_vote(self.player_id, bool(vote))
        game_phase = game_info.get("game_phase")
        if game_phase != GamePhase.VOTE:
            self.broadcast_after_vote_status(game_info, game_phase)
    
    def broadcast_after_vote_status(self, game_info, game_phase):
        vote_result_info_event = self._create_vote_result_object(game_info)
        async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, vote_result_info_event)
        if game_phase == GamePhase.PROPOSAL:
            async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, {"type": "send_new_proposal_info"})
            async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, {"type": "send_game_phase_update"})
        elif game_phase == GamePhase.MISSION:
            async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, {"type": "send_mission_info"})
            async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, {"type": "send_game_phase_update"})

    def _create_vote_result_object(self, game_info):
        event_info = dict()
        event_info["type"] = "send_vote_results"
        event_info["num_upvotes"] = game_info.get("num_upvotes")
        event_info["num_downvotes"] = game_info.get("num_downvotes")
        event_info["proposal_vote_info"] = game_info.get("proposal_vote_info")
        event_info["vote_maeved"] = game_info.get("vote_maeved")
        return event_info
        
    def send_mission_info(self, _):
        mission_info = self.game.get_mission_info()
        session_ids = mission_info.get("mission_session_ids")
        players_on_mission = mission_info.get("mission_players")
        isOnMission = self.player_id in session_ids
        game_phase = self.game.game_phase.value
        response = responses.MissionInfoResponse(game_phase, players_on_mission, isOnMission)
        self.send(response.serialize())

    def play_card(self, card_data):
        card_played = card_data.get("playedCard")
        after_mission_info = self.game.play_mission_card(self.player_id, MissionCard(card_played))
        self.broadcast_after_mission_info(after_mission_info)

    def broadcast_after_mission_info(self, after_mission_info):
        game_phase = after_mission_info.get("game_phase")
        if game_phase == GamePhase.MISSION:
            return
        async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, {"type": "send_mission_results"})
        async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, {"type": "send_game_phase_update"})
        if game_phase == GamePhase.PROPOSAL:
            async_to_sync(self.channel_layer.group_send)(self.lobby_group_name, {"type": "send_new_proposal_info"})


    def send_mission_results(self, _):
        prior_mission_num = self.game.mission_num - 1
        mission_result_info = self.game.get_all_mission_results().get(prior_mission_num)
        mission_result_info["priorMissionNum"] = prior_mission_num
        response = responses.MissionResultResponse(mission_result_info)
        self.send(response.serialize())

    def send_game_phase_update(self, _):
        game_phase = self.game.game_phase.value
        response = responses.GamePhaseChangeResponse(game_phase)
        self.send(response.serialize())

    def no_op(self, _):
        pass

    def use_ability(self):
        pass
