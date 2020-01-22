from abc import ABC, abstractmethod
from typing import Dict


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
                 player_number: int = 0,
                 player_name: str = ""):
        super().__init__(message_type, success, error_message)
        self.player_number = player_number
        self.player_name = player_name

    def _send_core(self, object_dict):
        object_dict["player_number"] = self.player_number
        object_dict["player_name"] = self.player_name
        return object_dict
