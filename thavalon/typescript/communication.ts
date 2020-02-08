namespace ThavalonWeb.Communication {
    export class WebSocketManager {
        private readonly _webSocket: WebSocket;
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
            if (this._webSocket.readyState !== WebSocket.OPEN) {
                throw new Error("The connection could not be established.");
            }

            // Connection opened successfully. Set up the required functions.
            this._webSocket.onmessage = this.parseIncomingMessage;
            this._webSocket.onerror = this.errorMessageHandler;
        }
        //#endregion

        //#region public methods

        //#endregion
        //#region private methods
        private parseIncomingMessage(rawMessage: MessageEvent): void {

        }

        private errorMessageHandler(rawMessage: MessageEvent): void {

        }
        //#endregion
    }
}