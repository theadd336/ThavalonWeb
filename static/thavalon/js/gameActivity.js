import { WebSocketManager } from "./communication.js";
import { GameView } from "./gameView.js";
export class GameDirector {
    //#endregion
    //#region Public Properties
    //#endregion
    //#region public methods
    constructor() {
        this._webSocketManager = new WebSocketManager();
        this._gameView = new GameView();
        this.initializeInfoOnConnect();
    }
    //#endregion
    //#region private methods
    initializeInfoOnConnect() {
        if (!this._webSocketManager.IsReady) {
            setTimeout(this.initializeInfoOnConnect, 200);
        }
        this._webSocketManager.send(JSON.stringify({ "type": "on_connect" }));
    }
}
//# sourceMappingURL=gameActivity.js.map