enum WEBSOCKET_READYSTATE {
    CONNECTING = 0,
    OPEN = 1,
    CLOSING = 2,
    CLOSED = 3,
};


interface PingResponse {
    "result": boolean,
    "errorMessage": string, // only populated if result is false
}

export class GameSocket {
    // the instantiated instance of the gamesocket
    private static instance: GameSocket;
    // the instantiated instance of the underlying websocket
    private static websocket: WebSocket;

    // functions for handling incoming websocket events
    private static socketOnOpen(event: Event) {
        console.log("Successfully initiated connection.");
    }

    private static socketOnMessage(event: MessageEvent) {
        console.log(event);
        console.log("Received message: " + event.data);
    }

    private static socketOnClose(event: CloseEvent) {
        console.log("Recieved on close message.");
    }

    private static socketOnError(event: Event) {
        console.log("Received on error message.");
    }

    /**
     * Get the account manager instance, creating one if needed.
     * 
     * @param socketUrl The url to connect to for the websocket
     * @returns The instance of the AccountManager.
     */
    public static getInstance(socketUrl: string): GameSocket {
        if (!GameSocket.instance) {
            this.instance = new GameSocket();
            this.websocket = new WebSocket(socketUrl);
            this.websocket.onopen = this.socketOnOpen;
            this.websocket.onmessage = this.socketOnMessage;
            this.websocket.onclose = this.socketOnClose
            this.websocket.onerror = this.socketOnError;
        }
        return GameSocket.instance;
    }

    public sendPing(): boolean {
        console.log("sending ping");
        if (GameSocket.websocket.readyState !== WEBSOCKET_READYSTATE.OPEN) {
            return false;
        }
        console.log("sent");
        GameSocket.websocket.send(JSON.stringify({
            "message_type": "ping",
        }));
        return true;
    }
}
