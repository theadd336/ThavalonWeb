import * as constants from "../Core/gameConstants.js";
import { WebSocketManager } from "./communication.js";
// import { GameView } from "./gameView.js";

export class GameDirector {
    //#region private members
    private _gamePhase: constants.GamePhase;
    private _role: string;
    private readonly _webSocketManager: WebSocketManager;
    // private readonly _gameView: GameView;
    //#endregion
    //#region Public Properties
    //#endregion

    //#region public methods
    constructor() {
        this._webSocketManager = new WebSocketManager();
        // this._gameView = new GameView();
        this.initializeInfoOnConnect();
        }
    
    //#endregion
    
    //#region private methods
    private initializeInfoOnConnect() {
        if (!this._webSocketManager.IsReady) {
            setTimeout(this.initializeInfoOnConnect, 200);
        }
        this._webSocketManager.send(JSON.stringify({"type": "on_connect"}))
    }
    //#endregion
}