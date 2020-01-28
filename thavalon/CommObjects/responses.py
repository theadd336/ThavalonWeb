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
        object_dict["error_message"] = self.error_message
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
