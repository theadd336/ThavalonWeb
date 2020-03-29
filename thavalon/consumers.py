from channels.generic.websocket import WebsocketConsumer
from asgiref.sync import async_to_sync
import json
from game.gamemanager import GameManager
from .CommObjects import responses
from game.game_constants import MissionCard, GamePhase
from enum import Enum
from typing import Any, Dict, Union, List

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
    AbilityInformationRequest = 9
    UseAbility = 10


class ChatConsumer(WebsocketConsumer):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.room_name = ""
        self.room_group_name = ""

    def connect(self):
        self.room_name = self.scope["url_route"]["kwargs"]["room_name"]
        self.room_group_name = "chat_%s" % self.room_name

        # Join room group
        async_to_sync(self.channel_layer.group_add)(
            self.room_group_name, self.channel_name
        )

        self.accept()

    def disconnect(self, close_code):
        # Leave room group
        async_to_sync(self.channel_layer.group_discard)(
            self.room_group_name, self.channel_name
        )

    # Receive message from WebSocket
    def receive(self, text_data):
        text_data_json = json.loads(text_data)
        message = text_data_json["message"]

        # Send message to room group
        async_to_sync(self.channel_layer.group_send)(
            self.room_group_name, {"type": "chat_message", "message": message}
        )

    # Receive message from room group
    def chat_message(self, event):
        message = event["message"]

        # Send message to WebSocket
        self.send(text_data=json.dumps({"message": message}))


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
            "start_game": self.start_game,
        }

    def connect(self):
        self.game_id = self.lobby_group_name = self.scope["url_route"]["kwargs"][
            "game_id"
        ]
        try:
            self.game = _GAME_MANAGER.get_game(self.game_id)
            self.player_id = self.scope["session"]["player_id"]
        except ValueError:
            return

        if not self.game.is_player_in_game(self.player_id):
            return
        async_to_sync(self.channel_layer.group_add)(
            self.lobby_group_name, self.channel_name
        )
        self.accept()
        self.join_game({"player_names": self.game.get_player_names_in_game()})
        return

    def disconnect(self, code):
        if code != 4000:
            self.game.remove_player(self.player_id)
        async_to_sync(self.channel_layer.group_discard)(
            self.lobby_group_name, self.channel_name
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
        async_to_sync(self.channel_layer.group_send)(
            self.lobby_group_name, joining_player_data
        )
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
            response.error_message = "An error occurred while leaving the game: " + str(
                error
            )
            self.send(text_data=response.serialize())
            return
        # Player has been removed. Now, tell everyone listening to remove the player from the table
        response.success = True
        response.player_number = player_number
        response.player_name = player_name
        async_to_sync(self.channel_layer.group_send)(
            self.lobby_group_name, response.serialize()
        )
        return

    def on_player_leave(self, event):
        # Pass leaving player information to the listening clients
        self.send(text_data=json.dumps(event))

    def start_game(self, _):
        try:
            self.game.start_game()
        except ValueError:
            return
        async_to_sync(self.channel_layer.group_send)(
            self.lobby_group_name, {"type": "on_start_game"}
        )
        return

    def on_start_game(self, event):
        self.scope["session"]["game_id"] = self.game_id
        self.scope["session"].save()
        self.send(text_data=json.dumps(event))


class GameConsumer(WebsocketConsumer):
    """Consumer class inheriting from WebSocketConsumer that handles all game-related messages."""

    def __init__(self, *args, **kwargs):
        """Initializes the consumer class. Called by Django framework.
        """
        super().__init__(*args, **kwargs)

        # Set up the instance variables and incoming message types.
        # Incoming message types are definined in the module IncomingMessageTypes enum.
        self.lobby_group_name = ""
        self.game_id = ""
        self.player_id = ""
        self.game = None
        self._message_types = {
            IncomingMessageTypes.RoleInformation.value: self.role_information,
            IncomingMessageTypes.SubmitVote.value: self.submit_vote,
            IncomingMessageTypes.AllMissionInfoRequest.value: self.send_all_mission_info,
            IncomingMessageTypes.SubmitProposal.value: self.broadcast_tentative_proposal,
            IncomingMessageTypes.MoveToVote.value: self.submit_proposal,
            IncomingMessageTypes.SubmitAssassination.value: self.no_op,
            IncomingMessageTypes.PlayerOrder.value: self.send_player_order,
            IncomingMessageTypes.ProposalVoteInformationRequest.value: self.send_proposal_vote_info,
            IncomingMessageTypes.PlayCard.value: self.play_card,
            IncomingMessageTypes.AbilityInformationRequest.value: self.send_ability_information,
            IncomingMessageTypes.UseAbility.value: self.use_ability,
        }

    def connect(self):
        """Called when a web socket connection is opening."""

        # Get the game ID from the connection.
        self.game_id = self.lobby_group_name = self.scope["url_route"]["kwargs"][
            "game_id"
        ]
        # Try joining the game. Both the game and the player must exist.
        # If the game doesn't exist or the player ID isn't valid, return.
        try:
            self.game = _GAME_MANAGER.get_game(self.game_id)
            self.player_id = self.scope["session"]["player_id"]
        except ValueError:
            return

        # We have a game and a player ID. Check to see if the player is registered.
        # If not, reject the connection.
        if not self.game.is_player_in_game(self.player_id):
            return

        # We have a valid player and game. Accept the connection and tell all clients.
        self.accept()
        async_to_sync(self.channel_layer.group_add)(
            self.lobby_group_name, self.channel_name
        )
        return

    def disconnect(self, code: int):
        """Called when a web socket connection is closing. 
        
        Parameters
        ----------
        code : int
            Code from the client. Currently unused.
        """
        async_to_sync(self.channel_layer.group_discard)(
            self.lobby_group_name, self.channel_name
        )

    def receive(self, text_data: str):
        """Receives a web socket message from the client.
            Initial entry point for all client messages.
        
        Parameters
        ----------
        text_data : string (JSON encoded)
            Serialized JSON string from the client.
        
        Raises
        ------
        NotImplementedError
            Error if the client's message type matches no methods in the class.
        """
        # Convert to a dictionary and parse the message's type.
        text_data = json.loads(text_data)
        message_type = text_data["type"]

        # Look up the function to call.
        # If there is no corresponding function, throw an error.
        # Otherwise, call the function with the text data.
        function_to_call = self._message_types.get(message_type)
        if function_to_call is None:
            raise NotImplementedError
        function_to_call(text_data)

    def role_information(self, _: Any):
        """Looks up role information from the game, given the current player. 
            Sends the information back to the client.
        
        Parameters
        ----------
        _ : Any
            Unused. Needed to match method signature of base class.
        """

        # Try looking up the player information.
        # If the lookup fails, parse the error to send to the client.
        success = True
        error_message = ""
        try:
            player_info = self.game.get_player_info(self.player_id)
        except ValueError as e:
            success = False
            error_message = str(e)
            player_info = None

        # Populate the response object with information and send.
        response = responses.RoleInformationResponse(
            success, error_message, player_info
        )
        self.send(response.serialize())

    def send_player_order(self, _: Any):
        """Sends the current player order to the client.
        
        Parameters
        ----------
        _ : Any
            Unused. Needed to match method signature of base class.
        """

        # Initialize variables and try to get current proposal information.
        # If the lookup fails, parse the error message for return to the client.
        success = True
        proposal_info = dict()
        error_message = ""
        try:
            proposal_info = self.game.get_proposal_info()
        except ValueError as e:
            success = False
            error_message = str(e)

        # Populate the response object and send it.
        response = responses.PlayerOrderResponse(
            success, error_message, proposal_info.get("proposal_order")
        )
        self.send(response.serialize())

    def send_vote_results(self, proposal_info: Dict[str, Union[int, bool]]):
        """Sends the results of the previous voting round.
            Formats the raw information from the game and adds
            Maeve-specific info.
        
        Parameters
        ----------
        proposal_info : Dict[str: int | Vote | bool]
            Dictionary of vote information.
            {
                [player_name: str]: int,
                num_upvotes: int,
                num_downvotes: int,
                was_maeved: bool
            }
        
        Raises
        ------
        TypeError
            Error if either num_upvotes or num_downvotes is not an int.
        """

        # Get the number of upvotes and downvotes. If this isn't populated,
        # bail out.
        num_upvotes = proposal_info.get("num_upvotes")
        num_downvotes = proposal_info.get("num_downvotes")
        if num_upvotes is None or num_downvotes is None:
            raise TypeError("Votes cannot be None.")

        # All required information is present, so extract remaining info from the dictionary.
        vote_info = proposal_info.get("proposal_vote_info")
        was_maeved = proposal_info.get("vote_maeved")
        last_proposal_number = self.game.current_proposal_num
        if last_proposal_number < 1:
            last_proposal_number = 1

        # Populate the vote_info array. This is tightly coupled with the client.
        # Then send the information.
        vote_info["Number of Upvotes"] = num_upvotes
        vote_info["Number of Downvotes"] = num_downvotes
        mission_number = self.game.mission_num
        response = responses.VoteResultMessage(
            mission_number, last_proposal_number, was_maeved, vote_info
        )
        self.send(response.serialize())

    def send_all_mission_info(self, _: Any):
        """Sends all mission info to the client. 
            Only used at the start of the game and on reconnect.
        
        Parameters
        ----------
        _ : Any
            Unused. Needed to match method signature of base class.
        """

        # Get all mission results and all mission default info from the game.
        # Then, update the default info with mission results.
        all_mission_results = self.game.get_all_mission_results()
        all_mission_info = self.game.get_all_mission_default_info()
        all_mission_info.update(all_mission_results)

        # Current information is in a dict, and the client needs a list,
        # Extract all information from the dict, add the mission number, and
        # append it to a list.
        all_mission_info_list = list()
        for mission_num, result_dict in all_mission_info.items():
            result_dict["missionNum"] = mission_num
            all_mission_info_list.append(result_dict)

        # Populate the response object and send to the server.
        response = responses.AllMissionInfoResponse(all_mission_info_list)
        self.send(response.serialize())

    def broadcast_tentative_proposal(
        self, proposal_info: Dict[str, Union[str, List[str]]]
    ):
        """Receives the tentative proposal information from the proposer
            and broadcasts it all clients.
        
        Parameters
        ----------
        proposal_info : Dict[str, Union(str, List[str])]
            Dictionary containing message type and list of players on the proposal.
            {
                "type": str
                "proposal": List[str]
            }
        """

        # Create a new proposal event to inform all clients of the proposal.
        # Then, broadcast the proposal to all clients, including self.
        tentative_proposal_event = {
            "type": "send_tentative_proposal",
            "proposal": proposal_info.get("proposal"),
        }
        async_to_sync(self.channel_layer.group_send)(
            self.lobby_group_name, tentative_proposal_event
        )

    def send_tentative_proposal(
        self, tentative_proposal_event: Dict[str, Union[str, List[str]]]
    ):
        """Sends a tentative proposal to the client.
        
        Parameters
        ----------
        tentative_proposal_event : Dict[str, Union(str, List[str])]
            Dictionary containing event type and proposal.
            {
                "type": str,
                "proposal": List[str]
            }
        """

        # Get the proposal information from the dictionary.
        # Populate the response object and send.
        proposal = tentative_proposal_event.get("proposal")
        response = responses.TentativeProposalResponse(proposal)
        self.send(response.serialize())

    def send_proposal_vote_info(self, _: Any):
        """Sends proposal or voting information. 
        Only used on reconnect or initial connection.
        
        Parameters
        ----------
        _ : Any
            Unused
        """

        # Get the game phase. If it's proposal (99% of the time),
        # call to send new proposal information.
        # If it's voting, send voting information.
        game_phase = self.game.game_phase
        if game_phase == GamePhase.PROPOSAL:
            self.send_new_proposal_info(None)
        elif game_phase == GamePhase.VOTE:
            self.send_vote_info(None)
        # TODO: Support assassination.
        elif game_phase == GamePhase.ASSASSINATION:
            pass

    def send_new_proposal_info(self, _: Any):
        """Sends new proposal information to the client.
        
        Parameters
        ----------
        _ : Any
            Unused
        """
        # Get the proposal information from the server and the proposer ID.
        proposal_info = self.game.get_proposal_info()
        proposer_id = proposal_info.get("proposer_id")

        # Populate the response object with information and send.
        response = responses.NewProposalResponse(
            self.game.get_player(proposer_id).name,
            self.player_id == proposer_id,
            proposal_info.get("proposal_size"),
            proposal_info.get("current_proposal_num"),
            proposal_info.get("max_num_proposers"),
        )
        self.send(response.serialize())

    def submit_proposal(self, proposal: Dict[str, Union[int, List[str]]]):
        """Submits a proposal from the client.
        
        Parameters
        ----------
        proposal : Dict[str, Union(str, List[str])
            The proposal information from the client.
            {
                "type": int,
                "proposal": List[str]
            }
        """
        proposal_list = proposal.get("proposal")
        try:
            self.game.set_proposal(proposal_list)
        except ValueError as e:
            # TODO: Support error messages for invalid values.
            print(e)
            return
        self.broadcast_moving_to_vote()

    def broadcast_moving_to_vote(self):
        """Alerts all other GameConsumer instances in the game that the 
        proposer has set the proposal. What happens next depends 
        on game phase.
        
        Parameters
        ----------
        game_info : Dict[str, Union(str, List[str])]
            Dictionary containing event type and proposal.
            {
                "game_phase": GamePhase,
            }
        """
        # Get the game phase from the game. What happens next depends on phase.
        game_phase = self.game.game_phase

        # If the game phase is still proposal (mission 1 only), tell all clients
        # to send the next proposal info.
        if game_phase == GamePhase.PROPOSAL:
            async_to_sync(self.channel_layer.group_send)(
                self.lobby_group_name, {"type": "send_new_proposal_info"}
            )

        # If it's force, tell all clients to move to mission immediately.
        elif game_phase == GamePhase.MISSION:
            async_to_sync(self.channel_layer.group_send)(
                self.lobby_group_name, {"type": "send_mission_info"}
            )
            async_to_sync(self.channel_layer.group_send)(
                self.lobby_group_name, {"type": "send_game_phase_update"}
            )

        # Otherwise, it's voting. Tell all clients to send vote info.
        else:
            async_to_sync(self.channel_layer.group_send)(
                self.lobby_group_name, {"type": "send_vote_info"}
            )
            async_to_sync(self.channel_layer.group_send)(
                self.lobby_group_name, {"type": "send_game_phase_update"}
            )

    def send_vote_info(self, _: Any):
        """Sends current voting information to the client.
        
        Parameters
        ----------
        _ : Any
            Unused.
        """

        # Get the latest proposal. Populate response object. Send it.
        proposal = self.game.current_proposals[-1]
        response = responses.MoveToVoteResponse(proposal)
        self.send(response.serialize())

    def submit_vote(self, vote_info: Dict[str, Union[str, int]]):
        """Submits a vote from the client. If it is the last vote, tell all 
        GameConsumers that the game phase is advancing.
        
        Parameters
        ----------
        vote_info : Dict[str, Union(str, int)]
            {
                "type": str,
                "vote": int
            }
        """

        # Get the vote from the client and try to submit it.
        vote = vote_info.get("vote")
        try:
            game_info = self.game.set_vote(self.player_id, bool(vote))
        except ValueError as e:
            # TODO: Handle errors here better.
            print(e)
            return

        # If successful, check the new game phase. If the game phase has changed,
        # tell all GameConsumer instances this.
        game_phase = game_info.get("game_phase")
        if game_phase != GamePhase.VOTE:
            self.broadcast_after_vote_status(game_info, game_phase)

    def broadcast_after_vote_status(
        self, game_info: Dict[str, Union[str, int, bool, Dict]], game_phase: GamePhase
    ):
        """Alerts all other GameConsumer instances in the game that the 
        vote has finished. What happens next depends on game phase.
        
        Parameters
        ----------
        game_info : Dict[str, Union(str, int, bool, Dict)]
            Game info object for voting information.
        game_phase : GamePhase
            Current game phase of the game.
        """

        # Create the dictionary for vote results and send it to all clients.
        # This should occur after every vote.
        vote_result_info_event = self._create_vote_result_object(game_info)
        async_to_sync(self.channel_layer.group_send)(
            self.lobby_group_name, vote_result_info_event
        )

        # If the game phase is proposal, then send new proposal information
        # and game phase change to all clients.
        if game_phase == GamePhase.PROPOSAL:
            async_to_sync(self.channel_layer.group_send)(
                self.lobby_group_name, {"type": "send_new_proposal_info"}
            )
            async_to_sync(self.channel_layer.group_send)(
                self.lobby_group_name, {"type": "send_game_phase_update"}
            )

        # If the game phase is mission, then send mission information
        # and game phase change to all clients.
        elif game_phase == GamePhase.MISSION:
            async_to_sync(self.channel_layer.group_send)(
                self.lobby_group_name, {"type": "send_mission_info"}
            )
            async_to_sync(self.channel_layer.group_send)(
                self.lobby_group_name, {"type": "send_game_phase_update"}
            )

    def _create_vote_result_object(
        self, game_info: Dict[str, Union[str, int, bool, Dict]]
    ) -> Dict[str, Union[str, int, bool, List[int]]]:
        """Converts the game info dictionary into an event to send to other consumers.
        
        Parameters
        ----------
        game_info : Dict[str, Union(str, int, bool, Dict)]
            Game info dict returned by setting a vote that advances game phase.
        
        Returns
        -------
        Dict[str, Union(str, int, bool, List[int])]
            Event dictionary to send to all consumers to update voting info.
        """

        # Unpack the game info dictionary and create and event dict to send.
        event_info = dict()
        event_info["type"] = "send_vote_results"
        event_info["num_upvotes"] = game_info.get("num_upvotes")
        event_info["num_downvotes"] = game_info.get("num_downvotes")
        event_info["proposal_vote_info"] = game_info.get("proposal_vote_info")
        event_info["vote_maeved"] = game_info.get("vote_maeved")
        return event_info

    def send_mission_info(self, _: Any):
        """Sends the current mission info to the client.
        
        Parameters
        ----------
        _ : Any
            Unused
        """

        # Get the mission info, session IDs, and players on the mission from the game.
        mission_info = self.game.get_mission_info()
        session_ids = mission_info.get("mission_session_ids")
        players_on_mission = mission_info.get("mission_players")

        # Get derived values such as is_on_mission and game phase.
        # Populate response object and send.
        is_on_mission = self.player_id in session_ids
        game_phase = self.game.game_phase.value
        response = responses.MissionInfoResponse(
            game_phase, players_on_mission, is_on_mission
        )
        self.send(response.serialize())

    def play_card(self, card_data: Dict[str, int]):
        """Plays a card submitted by the client.
        
        Parameters
        ----------
        card_data : Dict[str, int]
            Data from the client on the card played.
            {
                type: int,
                playedCard: int
            }
        """

        # Get the card played and try to submit it.
        card_played = card_data.get("playedCard")
        try:
            after_mission_info = self.game.play_mission_card(
                self.player_id, MissionCard(card_played)
            )
        except ValueError as e:
            error_response = responses.MissionInfoResponse(
                GamePhase.MISSION.value, [], True, success=False, error_message=str(e)
            )
            self.send(error_response.serialize())
            return

        # If the mission phase has ended, tell all consumers that fact.
        if after_mission_info.get("game_phase") != GamePhase.MISSION:
            self.broadcast_after_mission_info()

    def broadcast_after_mission_info(self, game_phase: GamePhase = None):
        """Alerts all other GameConsumer instances in the game that the 
        mission has finished. What happens next depends on game phase.
        
        Parameters
        ----------
        game_phase : GamePhase
            Current game phase. If not passed, will be looked up from game.
        """

        # Look up the game phase if it's not passed.
        if game_phase is None:
            game_phase = self.game.game_phase

        # If the game phase is mission, nothing to do, so return.
        if game_phase == GamePhase.MISSION:
            return

        # If it's not mission, then a mission happened, so tell all consumers
        # the mission results and that the game phase has changed.
        async_to_sync(self.channel_layer.group_send)(
            self.lobby_group_name, {"type": "send_mission_results"}
        )
        async_to_sync(self.channel_layer.group_send)(
            self.lobby_group_name, {"type": "send_game_phase_update"}
        )

        # If we're back in proposal, tell all consumers to send new proposal info.
        # TODO: Support Assassination
        if game_phase == GamePhase.PROPOSAL:
            async_to_sync(self.channel_layer.group_send)(
                self.lobby_group_name, {"type": "send_new_proposal_info"}
            )

    def send_mission_results(self, _: Any):
        """Sends the results of the last mission to the client.
        
        Parameters
        ----------
        _ : Any
            Unused
        """

        # Get the prior mission number and the mission result info from the server.
        # NOTE: This dictionary from game is tightly coupled to the client
        # and should be changed.
        prior_mission_num = self.game.mission_num - 1
        mission_result_info = self.game.get_all_mission_results().get(prior_mission_num)
        mission_result_info["priorMissionNum"] = prior_mission_num

        # Populate the response and send it.
        response = responses.MissionResultResponse(mission_result_info)
        self.send(response.serialize())

    def send_game_phase_update(self, _: Any):
        """Sends a game phase update to the client.
        
        Parameters
        ----------
        _ : Any
            Unused
        """

        # Get the game phase, populate the response, and send it.
        game_phase = self.game.game_phase.value
        response = responses.GamePhaseChangeResponse(game_phase)
        self.send(response.serialize())
        self.send_ability_information(None)

    def no_op(self, _):
        pass

    def send_ability_information(self, _: Any) -> None:
        """Sends ability related information to the client.
        
        Parameters
        ----------
        _ : Any
            Unused
        """
        is_maeve = self.game.placeholder(self.player_id)
        response = responses.AbilityInformationResponse(
            "You are Maeve", "Obscure", True, False, False
        )
        self.send(response.serialize())

    def use_ability(self, ability_message: Dict[str, Union[str, int]]) -> None:
        """Takes ability data from the client and tries to activate an ability.
        If successful, will broadcaast this as a toast.
        
        Parameters
        ----------
        ability_message : Dict[str: Union[str, int]]
            Message from the client with ability data.
        """
        targetted_player = ability_message.get("player")
        new_vote = ability_message.get("vote")
        # Use abilitiy
        ability_response = {"message": "Maeve has obscured the voting."}
        self.broadcast_ability_toast(ability_response)

    def broadcast_ability_toast(self, ability_response: Dict[str, str]) -> None:
        """Tells all other GameConsumers that someone has used an ability.
        
        Parameters
        ----------
        ability_response : Dict[str, str]
            The response from the game with ability related information.
        """
        ability_response["type"] = "send_toast_notification"
        async_to_sync(self.channel_layer.group_send)(
            self.lobby_group_name, ability_response
        )

    def send_toast_notification(self, ability_event: Dict[str, str]) -> None:
        """Sends a toast notification to the client.
        ***NOTE***: Unlike other sends, this MUST be called with an ability event.
        
        Parameters
        ----------
        ability_event : Dict[str, str]
            Event dictionary containing the message for the toast notification.
        """
        message = ability_event.get("message")
        response = responses.ToastNotificationResponse(message)
        self.send(response.serialize())
