import { EventDispatcher, IEvent } from "strongly-typed-events";
import * as constants from "../Core/commConstants.js";

export class WebSocketManager implements constants.IConnectionManager {

    private readonly _webSocket: WebSocket;
    private _onSuccessfulMessage: EventDispatcher<WebSocketManager, constants.WebSocketMessage>
    get IsReady(): boolean {
        return this._webSocket.readyState === WebSocket.OPEN;
    }

    get onSuccessfulMessage(): IEvent<WebSocketManager, constants.WebSocketMessage> {
        return this._onSuccessfulMessage.asEvent();
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
        this._onSuccessfulMessage = new EventDispatcher<WebSocketManager, constants.WebSocketMessage>();
    }
    //#endregion

    //#region public methods
    send(message: string): void {
        this._webSocket.send(message);
    }

    //#endregion
    //#region private methods
    private parseIncomingMessage(rawMessage: MessageEvent): void {
        const messageData = JSON.parse(rawMessage.data.toString());
        if (!this.isValidMessageFormat(messageData)) {
            throw new Error("Could not parse WebSocket event data.");
        }
        
        if (!messageData.success) {
            // TODO: Add better error handling
            alert(messageData.errorMessage);
            return;
        }
    }

    private errorMessageHandler(rawMessage: Event): void {
        throw new Error("A socket error has occurred");
    }

    private raiseSuccessfulMessage(data: constants.WebSocketMessage): void {
        this._onSuccessfulMessage.dispatch(this, data);
    }


    private isValidMessageFormat(messageData: any): messageData is constants.WebSocketMessage {
        return (messageData as constants.WebSocketMessage) !== undefined;
    }
    //#endregion
}