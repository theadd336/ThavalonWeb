interface PingResponse {
    result: boolean,
    errorMessage: string, // only populated if result is false
}

/**
 * The class containing the underlying websocket for playing the game.
 * Will contain a websocket, that can either be retrieved or optionally made.
 */
export class GameSocket {
    // the underlying gamesocket instance
    private static instance: GameSocket;
    // the instantiated instance of the underlying websocket
    private static websocket: WebSocket;

    // create the underlying websocket in the constructor
    private constructor(socketUrl: string) {
        GameSocket.websocket = new WebSocket(socketUrl);
        GameSocket.websocket.onopen = this.socketOnOpen;
        GameSocket.websocket.onmessage = this.socketOnMessage;
        GameSocket.websocket.onclose = this.socketOnClose
        GameSocket.websocket.onerror = this.socketOnError;
    } 

    private logMessage() {
        console.log("It's open!");
    }

    // functions for handling incoming websocket events
    private socketOnOpen(event: Event) {
        console.log("Successfully initiated connection.");
        this.logMessage();
    }

    private socketOnMessage(event: MessageEvent) {
        console.log(event);
        console.log("Received message: " + event.data);
    }

    private socketOnClose(event: CloseEvent) {
        console.log("Recieved on close message.");
    }

    private socketOnError(event: Event) {
        console.log("Received on error message.");
    }

    public static getInstance(): GameSocket {
        if (GameSocket.instance === undefined) {
            throw new Error("Gamesocket does not exist");
        }
        return GameSocket.instance;
    }

    public static createInstance(socketUrl: string): GameSocket {
        // close existing socket if it's open
        if (GameSocket.instance !== undefined) {
            GameSocket.websocket.close();
        }
        GameSocket.instance = new GameSocket(socketUrl);
        return GameSocket.instance;
    }

    public sendPing(): boolean {
        console.log("sending ping");
        if (GameSocket.websocket.readyState !== WebSocket.OPEN) {
            return false;
        }
        console.log("sent");
        GameSocket.websocket.send(JSON.stringify({
            "message_type": "ping",
        }));
        return true;
    }
}
