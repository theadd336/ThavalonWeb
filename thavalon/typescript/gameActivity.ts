import * as constants from "./gameConstants";
import { WebSocketManager } from "./communication";
import { GameView } from "./gameView";

namespace ThavalonWeb.Game {
    export class GameDirector {
        //#region private members
        private _gamePhase: constants.GamePhase;
        private _role: string;
        private readonly _webSocketManager: WebSocketManager;
        private readonly _gameView: GameView;
        //#endregion
        //#region Public Properties
        //#endregion

        //#region public methods
        constructor() {
                this._webSocketManager = new WebSocketManager();
                this._gameView = new GameView();
            }
        

        
        //#endregion
        
        //#region private methods
        //#endregion

    }
}