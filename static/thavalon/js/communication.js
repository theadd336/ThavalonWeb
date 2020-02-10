"use strict";
var ThavalonWeb;
(function (ThavalonWeb) {
    var Communication;
    (function (Communication) {
        class WebSocketManager {
            constructor(webSocketUrl) {
                // If there isn't a url, try to pull it from the window location.
                if (typeof webSocketUrl === "undefined") {
                    let wsUrlPath = window.location.pathname.split("/");
                    wsUrlPath[wsUrlPath.length - 1] = "game";
                    webSocketUrl = "ws://" + window.location.host + "/" +
                        wsUrlPath.join("/");
                }
                // Now there should be a URL. Try to open a connnection.
                this._webSocket = new WebSocket(webSocketUrl);
                // if (this._webSocket.readyState !== WebSocket.OPEN) {
                //     throw new Error("The connection could not be established.");
                // }
                // Connection opened successfully. Set up the required functions.
                this._webSocket.onmessage = (e) => {
                    this.parseIncomingMessage(e);
                };
                this._webSocket.onerror = (e) => {
                    this.errorMessageHandler(e);
                };
            }
            //#endregion
            //#region public methods
            send(message) {
                this._webSocket.send(message);
            }
            //#endregion
            //#region private methods
            parseIncomingMessage(rawMessage) {
                const messageData = JSON.parse(rawMessage.data);
                if (!this.isValidMessageFormat(messageData)) {
                    throw new Error("Could not parse WebSocket event data.");
                }
                console.log(messageData.type);
                if (!messageData.success) {
                    // TODO: Add better error handling
                    alert(messageData.errorMessage);
                    return;
                }
                // Route to appropriate handlers. TODO: use enums here.
                switch (messageData.type) {
                    case "gamestate":
                        populateGameState(messageData);
                        break;
                    case "on_start_game":
                        onStartGame(messageData);
                        break;
                    case "on_propose":
                        onPropose(messageData);
                        break;
                    case "on_vote_start":
                        onMoveToVote(messageData);
                        break;
                    case "new_proposal":
                        newProposal(messageData);
                        break;
                    case "on_mission_start":
                        onMissionStart(messageData);
                        break;
                    case "on_mission_results":
                        onMissionResults(messageData);
                        break;
                    case "vote_still_in_progress":
                        onVoteStillInProgress(messageData);
                        break;
                    case "mission_still_in_progress":
                        missionStillInProgress(messageData);
                        break;
                }
            }
            errorMessageHandler(rawMessage) {
            }
            isValidMessageFormat(messageData) {
                return messageData !== undefined;
            }
        }
        Communication.WebSocketManager = WebSocketManager;
    })(Communication = ThavalonWeb.Communication || (ThavalonWeb.Communication = {}));
})(ThavalonWeb || (ThavalonWeb = {}));
//# sourceMappingURL=communication.js.map