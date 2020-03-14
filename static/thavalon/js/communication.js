import { EventDispatcher } from "../../node_modules/strongly-typed-events/dist/index";
export class WebSocketManager {
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
        this._webSocket.onmessage = (e) => {
            this.parseIncomingMessage(e);
        };
        this._webSocket.onerror = (e) => {
            this.errorMessageHandler(e);
        };
        // Set up event handlers for recieved messages
        this.initializeEventHandlers();
    }
    get IsReady() {
        return this._webSocket.readyState === WebSocket.OPEN;
    }
    //#endregion
    //#region public methods
    send(message) {
        this._webSocket.send(message);
    }
    // Message events from the web socket.
    get onStartGame() {
        return this._gameStartEvent.asEvent();
    }
    get onProposalReceived() {
        return this._proposalReceivedEvent.asEvent();
    }
    get onVoteStart() {
        return this._moveToVoteEvent.asEvent();
    }
    get onNewProposal() {
        return this._newProposalEvent.asEvent();
    }
    get onMissionStart() {
        return this._missionStartEvent.asEvent();
    }
    get onMissionResults() {
        return this._missionResultsEvent.asEvent();
    }
    get onVoteStillInProgress() {
        return this._voteStillInProgressEvent.asEvent();
    }
    get onMissionStillInProgress() {
        return this._missionStillInProgressEvent.asEvent();
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
    /**
     * Initializes event handlers upon instantiation.
     */
    initializeEventHandlers() {
        this._gameStartEvent = new EventDispatcher();
        this._proposalReceivedEvent = new EventDispatcher();
        this._moveToVoteEvent = new EventDispatcher();
        this._newProposalEvent = new EventDispatcher();
        this._missionStartEvent = new EventDispatcher();
        this._missionResultsEvent = new EventDispatcher();
        this._voteStillInProgressEvent = new EventDispatcher();
        this._missionStillInProgressEvent = new EventDispatcher();
    }
}
//# sourceMappingURL=communication.js.map