"use strict";
var WebSocketManager = ThavalonWeb.Communication.WebSocketManager;
var ThavalonWeb;
(function (ThavalonWeb) {
    var Game;
    (function (Game) {
        class GameDirector {
            //#endregion
            //#region Public Properties
            //#endregion 
            constructor(roleInformation, allMissionInfo, gamePhase = 0 /* Proposal */) {
                this._role = roleInformation.role;
                this._gamePhase = gamePhase;
                this._webSocketManager = new WebSocketManager();
            }
        }
        Game.GameDirector = GameDirector;
    })(Game = ThavalonWeb.Game || (ThavalonWeb.Game = {}));
})(ThavalonWeb || (ThavalonWeb = {}));
//# sourceMappingURL=gameDirector.js.map