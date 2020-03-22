import { EventDispatcher, IEvent } from "strongly-typed-events";
import * as constants from "../Core/commConstants";

export interface WebSocketProp {
    webSocket: WebSocketManager
}

export class WebSocketManager {

    private _webSocket: WebSocket;
    private _onSuccessfulMessage: EventDispatcher<WebSocketManager, constants.IncomingMessage>;
    private _onErrorMessage: EventDispatcher<WebSocketManager, constants.IncomingMessage>
    get IsOpen(): boolean {
        return (this._webSocket.readyState === WebSocket.OPEN);
    }

    get onSuccessfulMessage(): IEvent<WebSocketManager, constants.IncomingMessage> {
        return this._onSuccessfulMessage.asEvent();
    }

    get onErrorMessage(): IEvent<WebSocketManager, constants.IncomingMessage> {
        return this._onErrorMessage.asEvent();
    }

    //#region constructors
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
        this._onSuccessfulMessage = new EventDispatcher<WebSocketManager, constants.IncomingMessage>();
        this._onErrorMessage = new EventDispatcher<WebSocketManager, constants.IncomingMessage>();
    }
    //#endregion

    //#region public methods
    send(message: constants.OutgoingMessage): void {
        if (!(message.type in constants.OutgoingMessageTypes)) {
            //TODO: Improve this error.
            throw new Error("");
        }
        const serializedMessage = JSON.stringify(message);
        this.waitForOpenConnection(this, () => {
            this._webSocket.send(serializedMessage);
        })
    }
    
    /**
     * Holds messages until the connection is open. Then, calls the callback to send the message.
     * @param socketManager An instance of the websocket manager. Probably this.
     * @param callBack Callback function upon the connection opening.
     */
    waitForOpenConnection(socketManager: WebSocketManager, callBack: any) {
        setTimeout(
            function () {
                if (socketManager.IsOpen) {
                    if (callBack != null) {
                        callBack();
                    }
                } else {
                    socketManager.waitForOpenConnection(socketManager, callBack);
                }
            }, 3000 // Wait 3 seconds max.
        );
    }
    //#endregion
    //#region private methods
    private parseIncomingMessage(rawMessage: MessageEvent): void {
        const messageData = JSON.parse(rawMessage.data.toString());
        if (!this.isValidMessageFormat(messageData)) {
            throw new Error("Could not parse WebSocket event data.");
        }
        
        if (!messageData.success) {
            this.raiseErrorMessage(messageData);
        } else {
            this.raiseSuccessfulMessage(messageData);
        }
    }

    private errorMessageHandler(rawMessage: Event): void {
        throw new Error("A socket error has occurred");
    }

    private raiseSuccessfulMessage(data: constants.IncomingMessage): void {
        this._onSuccessfulMessage.dispatch(this, data);
    }

    private raiseErrorMessage(data: constants.IncomingMessage): void {
        this._onErrorMessage.dispatch(this, data);
    }


    private isValidMessageFormat(messageData: any): messageData is constants.IncomingMessage {
        return (messageData as constants.IncomingMessage) !== undefined;
    }
    //#endregion
}