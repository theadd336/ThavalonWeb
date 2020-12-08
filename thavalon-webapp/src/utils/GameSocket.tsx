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
    // can also be undefined since it's not explicitly set in constructor
    private websocket: WebSocket;

    // create the underlying websocket in the constructor
    private constructor(socketUrl: string) {
        this.websocket = new WebSocket(socketUrl);
        this.websocket.onopen = this.socketOnOpen.bind(this);
        this.websocket.onmessage = this.socketOnMessage.bind(this);
        this.websocket.onclose = this.socketOnClose.bind(this);
        this.websocket.onerror = this.socketOnError.bind(this);
    } 
    // functions for handling incoming websocket events
    private socketOnOpen(event: Event) {
        console.log("Successfully initiated connection.");
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
            GameSocket.instance.websocket?.close();
        }
        GameSocket.instance = new GameSocket(socketUrl);
        return GameSocket.instance;
    }

    public sendPing(): boolean {
        console.log("sending ping");
        if (GameSocket.instance.websocket?.readyState !== WebSocket.OPEN) {
            return false;
        }
        console.log("sent");
        GameSocket.instance.websocket?.send(JSON.stringify({
            message_type: "ping",
        }));
        return true;
    }
}
