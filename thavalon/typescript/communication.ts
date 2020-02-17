import { EventDispatcher, IEvent } from "../../node_modules/strongly-typed-events/dist/index";
import * as constants from "./commConstants.js";

declare function populateGameState(data: constants.WebSocketMessage): void
declare function onStartGame(data: constants.WebSocketMessage): void
declare function onPropose(data: constants.WebSocketMessage): void
declare function onMoveToVote(data: constants.WebSocketMessage): void
declare function newProposal(data: constants.WebSocketMessage): void
declare function onMissionStart(data: constants.WebSocketMessage): void
declare function onMissionResults(data: constants.WebSocketMessage): void
declare function onVoteStillInProgress(data: constants.WebSocketMessage): void
declare function missionStillInProgress(data: constants.WebSocketMessage): void

export class WebSocketManager {
    private readonly _webSocket: WebSocket;
    private _gameStartEvent: EventDispatcher<WebSocketManager, string>;
    private _proposalReceivedEvent: EventDispatcher<WebSocketManager, constants.ProposalReceivedMessage>;
    private _moveToVoteEvent: EventDispatcher<WebSocketManager, constants.MoveToVoteMessage>;
    private _newProposalEvent: EventDispatcher<WebSocketManager, constants.NewProposalMessage>;
    private _missionStartEvent: EventDispatcher<WebSocketManager, constants.MissionStartMessage>;
    private _missionResultsEvent: EventDispatcher<WebSocketManager, constants.MissionResultsMessage>;
    private _voteStillInProgressEvent: EventDispatcher<WebSocketManager, constants.VoteStillInProgressMessage>;
    private _missionStillInProgressEvent: EventDispatcher<WebSocketManager, constants.MissionStillInProgressMessage>;
    get IsReady() {
        return this._webSocket.readyState === WebSocket.OPEN;
    }
    //#region constructors
    constructor()
    constructor(webSocketUrl: string)
    constructor(webSocketUrl?: string) {
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
            this.parseIncomingMessage(e)
        };
        this._webSocket.onerror = (e) => {
            this.errorMessageHandler(e);
        };

        // Set up event handlers for recieved messages
        this.initializeEventHandlers();
    }
    //#endregion

    //#region public methods
    send(message: string): void {
        this._webSocket.send(message);
    }

    // Message events from the web socket.
    get onStartGame(): IEvent<WebSocketManager, string> {
        return this._gameStartEvent.asEvent();
    }

    get onProposalReceived(): IEvent<WebSocketManager, constants.ProposalReceivedMessage> {
        return this._proposalReceivedEvent.asEvent();
    }

    get onVoteStart(): IEvent<WebSocketManager, constants.MoveToVoteMessage> {
        return this._moveToVoteEvent.asEvent();
    }

    get onNewProposal(): IEvent<WebSocketManager, constants.NewProposalMessage> {
        return this._newProposalEvent.asEvent();
    }

    get onMissionStart(): IEvent<WebSocketManager, constants.MissionStartMessage> {
        return this._missionStartEvent.asEvent();
    }

    get onMissionResults(): IEvent<WebSocketManager, constants.MissionResultsMessage> {
        return this._missionResultsEvent.asEvent();
    }

    get onVoteStillInProgress(): IEvent<WebSocketManager, constants.VoteStillInProgressMessage> {
        return this._voteStillInProgressEvent.asEvent();
    }

    get onMissionStillInProgress(): IEvent<WebSocketManager, constants.MissionStillInProgressMessage> {
        return this._missionStillInProgressEvent.asEvent();
    }

    //#endregion
    //#region private methods
    private parseIncomingMessage(rawMessage: MessageEvent): void {
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

    private errorMessageHandler(rawMessage: Event): void {
    }

    private isValidMessageFormat(messageData: any): messageData is constants.WebSocketMessage {
        return (messageData as constants.WebSocketMessage) !== undefined;
    }

    /**
     * Initializes event handlers upon instantiation.
     */
    private initializeEventHandlers(): void {
        this._gameStartEvent = new EventDispatcher<WebSocketManager, string>();
        this._proposalReceivedEvent = new EventDispatcher<WebSocketManager, constants.ProposalReceivedMessage>();
        this._moveToVoteEvent = new EventDispatcher<WebSocketManager, constants.MoveToVoteMessage>();
        this._newProposalEvent = new EventDispatcher<WebSocketManager, constants.NewProposalMessage>();
        this._missionStartEvent = new EventDispatcher<WebSocketManager, constants.MissionStartMessage>();
        this._missionResultsEvent = new EventDispatcher<WebSocketManager, constants.MissionResultsMessage>();
        this._voteStillInProgressEvent = new EventDispatcher<WebSocketManager, constants.VoteStillInProgressMessage>();
        this._missionStillInProgressEvent = new EventDispatcher<WebSocketManager, constants.MissionStillInProgressMessage>();
    }
    //#endregion
}