from abc import ABC, abstractmethod
from typing import Dict, List


class Response(ABC):
    def __init__(self, message_type: str, success: bool = False, error_message: str = ""):
        self.type = message_type
        self.success = success
        self.error_message = error_message

    def send(self) -> Dict[str, object]:
        object_dict = dict()
        object_dict["type"] = self.type
        object_dict["success"] = self.success
        object_dict["errorMessage"] = self.error_message
        return self._send_core(object_dict)

    def _send_core(self, object_dict):
        return object_dict


class JoinLeaveGameResponse(Response):
    def __init__(self, message_type: str,
                 success: bool = False,
                 error_message: str = "",
                 player_names: List[str] = None,
                 player_list: List[str] = None):
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
        return object_dict


class NewProposalResponse(Response):
    def __init__(self):
        super().__init__(success=True, message_type="new_proposal")
        self.is_proposing = False
        self.proposer_index = 0
        self.proposal_order = None
        self.proposal_num = 1
        self.max_num_proposals = 0
        self.proposal_size = 1
        self.current_proposal = None

    def _send_core(self, object_dict):
        object_dict["isProposing"] = self.is_proposing
        object_dict["proposerIndex"] = self.proposer_index
        object_dict["proposalOrder"] = self.proposal_order
        object_dict["proposalNum"] = self.proposal_num
        object_dict["maxNumProposals"] = self.max_num_proposals
        object_dict["proposalSize"] = self.proposal_size
        object_dict["currentProposal"] = self.current_proposal
        return object_dict


class OnProposeResponse(Response):
    def __init__(self, proposed_player_list: List[str] = None):
        super().__init__(message_type="on_propose", success=True)
        self.proposed_player_list = proposed_player_list
        self.proposer_name = ""

    def _send_core(self, object_dict):
        object_dict["proposedPlayerList"] = self.proposed_player_list
        object_dict["proposerName"] = self.proposer_name
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

    def _send_core(self, object_dict):
        object_dict["playerList"] = self.player_list
        object_dict["submittedVote"] = self.submitted_vote
        object_dict["isOnMission"] = self.is_on_mission
        return object_dict


class OnMissionResultsResponse(Response):
    def __init__(self, message_type: str = ""):
        super().__init__(message_type=message_type, success=True)
        self.card_played = ""
        self.mission_result = 0
        self.prior_mission_num = 0
        self.played_cards = None

    def _send_core(self, object_dict):
        object_dict["cardPlayed"] = self.card_played
        object_dict["missionResult"] = self.mission_result
        object_dict["priorMissionNum"] = self.prior_mission_num
        object_dict["playedCards"] = self.played_cards
        return object_dict
