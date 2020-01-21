from abc import ABC, abstractmethod
from typing import Dict


class Response(ABC):
    def __init__(self, success: bool = False, error_message: str = ""):
        self.success = success
        self.error_message = error_message

    @abstractmethod
    def send(self):
        pass


class JoinGameResponse(Response):
    def __init__(self, success=0, error_message="", player_number=0):
        super().__init__(success, error_message)
        self.player_number = 0

    def send(self) -> Dict[str, int]:
        object_dict = dict()
        object_dict["success"] = self.success
        object_dict["error_message"] = self.error_message
        object_dict["player_number"] = self.player_number
        return object_dict
