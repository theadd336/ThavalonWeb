import * as constants from "./gameConstants";
import { WebSocketManager } from "./communication";

namespace ThavalonWeb.Game {
    export class GameDirector {
        //#region private members
        private _gamePhase: constants.GamePhase;
        private _role: string;
        private  readonly _webSocketManager: WebSocketManager;
        //#endregion
        //#region Public Properties
        //#endregion 
        constructor(
            roleInformation: constants.RoleInformation,
            allMissionInfo: constants.AllMissionInfo,
            gamePhase = constants.GamePhase.Proposal) {
                this._role = roleInformation.role;
                this._gamePhase = gamePhase;
                this._webSocketManager = new WebSocketManager();
            }
    }
}