from abc import ABC, abstractmethod
from typing import Dict, List, Any
from enum import Enum
import json


class OutgoingMessageTypes(Enum):
    RoleInformation = 0
    MissionResult = 1
    AllMissionInfo = 2
    PlayerOrder = 3
    VoteResult = 4
    NewProposal = 5
    ProposalReceived = 6
    MoveToVote = 7
    AssassinationResponse = 8
    MissionInformation = 9
    GamePhaseChange = 10
    AbilityInformationResponse = 11
    ToastNotification = 12


class Response(ABC):
    def __init__(
        self, message_type: int, success: bool = False, error_message: str = ""
    ):
        self.type = message_type
        self.success = success
        self.error_message = error_message

    def serialize(self) -> str:
        object_dict = dict()
        object_dict["type"] = self.type
        object_dict["success"] = self.success
        object_dict["errorMessage"] = self.error_message
        object_dict = self._send_core(object_dict)
        return json.dumps(object_dict)

    @abstractmethod
    def _send_core(self, object_dict):
        return object_dict


class JoinLeaveGameResponse(Response):
    def __init__(
        self,
        message_type: str,
        success: bool = False,
        error_message: str = "",
        player_names: List[str] = None,
        player_list: List[str] = None,
    ):
        super().__init__(message_type, success, error_message)
        self.player_names = player_names
        self.player_list = player_list

    def _send_core(self, object_dict):
        object_dict["player_names"] = self.player_names
        object_dict["player_list"] = self.player_list
        return object_dict


class GameStateResponse(Response):
    def __init__(self, success: bool = False, error_message: str = ""):
        super().__init__("gamestate", success, error_message)
        self.proposal_order = None
        self.mission_sizes = None
        self.mission_results = None
        self.role_information = ""
        self.mission_players = None
        self.proposer_index = None
        self.proposal_num = 1
        self.current_phase = None
        self.declarations = None
        self.last_vote_information = None
        self.is_proposing = False
        self.max_num_proposals = 1
        self.mission_num = 1
        self.current_proposal = None
        self.proposal_size = 0
        self.mission_info = None

    def _send_core(self, object_dict):
        object_dict["proposalOrder"] = self.proposal_order
        object_dict["missionSizes"] = self.mission_sizes
        object_dict["missionResults"] = self.mission_results
        object_dict["roleInformation"] = self.role_information
        object_dict["missionPlayers"] = self.mission_players
        object_dict["proposerIndex"] = self.proposer_index
        object_dict["proposalNum"] = self.proposal_num
        object_dict["currentPhase"] = self.current_phase
        object_dict["declarations"] = self.declarations
        object_dict["lastVoteInfo"] = self.last_vote_information
        object_dict["isProposing"] = self.is_proposing
        object_dict["maxNumProposals"] = self.max_num_proposals
        object_dict["missionNum"] = self.mission_num
        object_dict["currentProposal"] = self.current_proposal
        object_dict["proposalSize"] = self.proposal_size
        object_dict["missionInfo"] = self.mission_info
        return object_dict


class OnProposeResponse(Response):
    def __init__(self, proposed_player_list: List[str] = None):
        super().__init__(message_type="on_propose", success=True)
        self.proposed_player_list = proposed_player_list
        self.proposer_name = ""
        self.is_proposing = False

    def _send_core(self, object_dict):
        object_dict["proposedPlayerList"] = self.proposed_player_list
        object_dict["proposerName"] = self.proposer_name
        object_dict["isProposing"] = self.is_proposing
        return object_dict


class OnVoteStartResponse(Response):
    def __init__(self, player_list: List[str] = None):
        super().__init__(success=True, message_type="on_vote_start")
        self.player_list = player_list

    def _send_core(self, object_dict):
        object_dict["playerList"] = self.player_list
        return object_dict


class OnVoteResultsResponse(Response):
    def __init__(self, message_type: str = "", player_list: List[str] = None):
        super().__init__(success=True, message_type=message_type)
        self.player_list = player_list
        self.is_on_mission = False
        self.submitted_vote = False
        self.proposal_vote_info = None

    def _send_core(self, object_dict):
        object_dict["playerList"] = self.player_list
        object_dict["submittedVote"] = self.submitted_vote
        object_dict["isOnMission"] = self.is_on_mission
        object_dict["priorVoteInfo"] = self.proposal_vote_info
        return object_dict


class OnMissionResultsResponse(Response):
    def __init__(self, message_type: str = ""):
        super().__init__(message_type=message_type, success=True)
        self.card_played = ""
        self.mission_result = 0
        self.prior_mission_num = 0
        self.played_cards = None
        self.players_on_mission = None

    def _send_core(self, object_dict):
        object_dict["cardPlayed"] = self.card_played
        object_dict["missionResult"] = self.mission_result
        object_dict["priorMissionNum"] = self.prior_mission_num
        object_dict["playedCards"] = self.played_cards
        object_dict["playersOnMission"] = self.players_on_mission
        return object_dict


# Everything below here is legit.
class RoleInformationResponse(Response):
    def __init__(
        self, success=True, error_message="", player_info: Dict[str, Any] = None
    ):
        super().__init__(
            message_type=OutgoingMessageTypes.RoleInformation.value,
            success=success,
            error_message=error_message,
        )
        self.role = ""
        self.team = None
        self.description = ""
        if player_info is not None:
            self.role = player_info["role"]
            self.team = player_info["team"]
            self.description = player_info["description"]

    def _send_core(self, object_dict):
        local_dict = dict()
        local_dict["role"] = self.role
        local_dict["team"] = self.team
        local_dict["description"] = self.description
        object_dict["data"] = local_dict
        return object_dict


class PlayerOrderResponse(Response):
    def __init__(self, success=True, error_message="", player_order: List[str] = None):

        super().__init__(
            message_type=OutgoingMessageTypes.PlayerOrder.value,
            success=success,
            error_message=error_message,
        )
        self.player_order = player_order

    def _send_core(self, object_dict):
        object_dict["data"] = {"playerOrder": self.player_order}
        return object_dict


class VoteResultMessage(Response):
    def __init__(
        self,
        mission_number: int,
        proposal_number: int,
        was_maeved: bool,
        vote_result: Dict[str, int],
    ):
        super().__init__(OutgoingMessageTypes.VoteResult.value, True, "")

        self.mission_number = mission_number
        self.proposal_number = proposal_number
        self.vote_information = vote_result
        self.was_maeved = was_maeved

    def _send_core(self, object_dict):
        local_dict = dict()
        local_dict["missionNumber"] = self.mission_number
        local_dict["proposalNumber"] = self.proposal_number
        local_dict["voteInformation"] = self.vote_information
        local_dict["wasMaeved"] = self.was_maeved
        object_dict["data"] = local_dict
        return object_dict


class AllMissionInfoResponse(Response):
    def __init__(self, all_mission_info):
        super().__init__(OutgoingMessageTypes.AllMissionInfo.value, True)
        self.all_mission_info = all_mission_info
        self.num_missions = len(all_mission_info)

    def _send_core(self, object_dict):
        local_dict = {
            "missionsInfo": self.all_mission_info,
            "numMissions": self.num_missions,
        }
        object_dict["data"] = local_dict
        return object_dict


class TentativeProposalResponse(Response):
    def __init__(self, proposal: List[str]):
        super().__init__(OutgoingMessageTypes.ProposalReceived.value, True)
        self.proposal = proposal

    def _send_core(self, object_dict):
        local_dict = {"proposal": self.proposal}
        object_dict["data"] = local_dict
        return object_dict


class NewProposalResponse(Response):
    def __init__(
        self,
        proposer: str,
        is_proposing: bool,
        num_on_proposal: int,
        proposal_number: int,
        max_num_proposals: int,
    ):

        super().__init__(OutgoingMessageTypes.NewProposal.value, True)
        self.proposer = proposer
        self.is_proposing = is_proposing
        self.num_on_proposal = num_on_proposal
        self.proposal_number = proposal_number
        self.max_num_proposals = max_num_proposals

    def _send_core(self, object_dict):
        local_dict = dict()
        local_dict["proposer"] = self.proposer
        local_dict["isProposing"] = self.is_proposing
        local_dict["numOnProposal"] = self.num_on_proposal
        local_dict["proposalNumber"] = self.proposal_number
        local_dict["maxNumProposals"] = self.max_num_proposals
        object_dict["data"] = local_dict
        return object_dict


class MoveToVoteResponse(Response):
    def __init__(self, proposal: List[str]):
        super().__init__(OutgoingMessageTypes.MoveToVote.value, True)
        self.proposal = proposal

    def _send_core(self, object_dict):
        local_dict = {"proposal": self.proposal}
        object_dict["data"] = local_dict
        return object_dict


class MissionInfoResponse(Response):
    def __init__(
        self,
        game_phase: int,
        players_on_mission: List[str],
        isOnMission: bool,
        success=True,
        error_message="",
    ):
        super().__init__(
            OutgoingMessageTypes.MissionInformation.value, success, error_message
        )
        self.game_phase = game_phase
        self.players_on_mission = players_on_mission
        self.isOnMission = isOnMission

    def _send_core(self, object_dict):
        local_dict = dict()
        local_dict["gamePhase"] = self.game_phase
        local_dict["playersOnMission"] = self.players_on_mission
        local_dict["isOnMission"] = self.isOnMission
        object_dict["data"] = local_dict
        return object_dict


class MissionResultResponse(Response):
    def __init__(self, mission_result: Dict[str, Any]):

        super().__init__(OutgoingMessageTypes.MissionResult.value, True)
        self.mission_result = mission_result

    def _send_core(self, object_dict):
        object_dict["data"] = self.mission_result
        return object_dict


class GamePhaseChangeResponse(Response):
    def __init__(self, new_game_phase: int):
        super().__init__(OutgoingMessageTypes.GamePhaseChange.value, True)
        self.game_phase = new_game_phase

    def _send_core(self, object_dict):
        object_dict["data"] = {"gamePhase": self.game_phase}
        return object_dict


class AbilityInformationResponse(Response):
    """Serializable class representing ability information."""

    def __init__(
        self,
        description: str,
        caption: str,
        can_use_ability: bool,
        needs_player_list: bool = False,
        needs_vote_options: bool = False,
        ability_timeout: int = None,
    ):
        """Initializes the response object.
        
        Parameters
        ----------
        description : str
            Description of the ability
        caption : str
            Caption for the "Use Ability" button. If empty, "Use Ability" will be shown.
        can_use_ability : bool
            True if the ability can be used. False otherwise.
        needs_player_list : bool, optional
            Does the ability need to allow selecting players, by default False.
        needs_vote_options : bool, optional.
            Does the ability need to allow selecting votes, by default False.
        ability_timeout : int, optional
        Timeout for how long the ability should be allowed to be used, by default None
        """
        super().__init__(OutgoingMessageTypes.AbilityInformationResponse.value, True)
        self.description = description
        self.caption = caption
        self.can_use_ability = can_use_ability
        self.needs_player_list = needs_player_list
        self.needs_vote_options = needs_vote_options
        self.ability_timeout = ability_timeout

    def _send_core(self, object_dict: Dict[str, Any]):
        local_dict = dict()
        local_dict["description"] = self.description
        local_dict["caption"] = self.caption
        local_dict["canUseAbility"] = self.can_use_ability
        local_dict["needsPlayerList"] = self.needs_player_list
        local_dict["needsVoteOptions"] = self.needs_vote_options
        local_dict["abilityTimeout"] = self.ability_timeout
        object_dict["data"] = local_dict
        return object_dict


class ToastNotificationResponse(Response):
    """Serializable class representing a toast notification."""

    def __init__(self, message: str, subtype: int = None) -> None:
        """Initializes the toast notification response.
        
        Parameters
        ----------
        message : str
            Message to display
        subtype : int, optional
            Message subtype, by default None
        """
        super().__init__(OutgoingMessageTypes.ToastNotification.value, True)
        self.message = message

    def _send_core(self, object_dict: Dict[str, Any]) -> Dict[str, Any]:
        object_dict["data"] = {"message": self.message}
        return object_dict
