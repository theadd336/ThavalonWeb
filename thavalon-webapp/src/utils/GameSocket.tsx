import { ISimpleEvent, SimpleEventDispatcher } from "strongly-typed-events";

interface PingResponse {
    result: boolean,
    errorMessage: string, // only populated if result is false
}

export enum OutboundMessageType {
    Ping = "Ping",
    GetLobbyState = "GetLobbyState",
    GetPlayerList = "GetPlayerList",
    StartGame = "StartGame",
}

export interface OutboundMessage {
    messageType: OutboundMessageType,
    data?: object | string,
}

export enum InboundMessageType {
    Pong = "Pong",
    PlayerList = "PlayerList",
    LobbyState = "LobbyState",
}

export interface InboundMessage {
    messageType: InboundMessageType,
    data?: object | string | number,
}

/**
 * The class containing the underlying websocket for playing the game.
 * Will contain a websocket, that can either be retrieved or optionally made.
 */
export class GameSocket {
    // the underlying gamesocket instance
    private static instance: GameSocket | undefined;
    // the instantiated instance of the underlying websocket
    // can also be undefined since it's not explicitly set in constructor
    private websocket: WebSocket;
    // event handler for lobby events
    private _onLobbyEvent: SimpleEventDispatcher<InboundMessage>;

    /**
     * Construct the underlying websocket instance and set up function handlers.
     * @param socketUrl The socketUrl for the websocket
     */
    private constructor(socketUrl: string) {
        this.websocket = new WebSocket(socketUrl);
        this._onLobbyEvent = new SimpleEventDispatcher<InboundMessage>();
        this.websocket.onopen = this.socketOnOpen.bind(this);
        this.websocket.onmessage = this.socketOnMessage.bind(this);
        this.websocket.onclose = this.socketOnClose.bind(this);
        this.websocket.onerror = this.socketOnError.bind(this);
    }

    /**
     * Listen to websocket's onopen event.
     * @param event Event received when socket open.
     */
    private socketOnOpen(event: Event) {
        console.log("Successfully initiated connection.");
    }

    /**
     * Listen to websocket's onmessage event.
     * @param event Event received when socket gets message.
     */
    private socketOnMessage(event: MessageEvent) {
        console.log(event);
        console.log("Received message: " + event.data);
        const message = JSON.parse(event.data);
        this._onLobbyEvent.dispatch(message);
    }

    /**
     * Listen to websocket's onclose event.
     * @param event Event received when socket is closed.
     */
    private socketOnClose(event: CloseEvent) {
        console.log("Recieved on close message.");
    }

    /**
     * Listen to websocket's onerror event.
     * @param event Event received when socket errors.
     */
    private socketOnError(event: Event) {
        console.log("Received on error message.");
    }

    /**
     * Send a message on the websocket. This will wait until websocket is
     * open before sending the message.
     * @param outboundMessage The outboundMessage to be sent to the server. 
     */
    public sendMessage(outboundMessage: OutboundMessage) {
        if (this.websocket.readyState === WebSocket.OPEN) {
            console.log("Sending message of type: " + outboundMessage.messageType);
            this.websocket.send(JSON.stringify(outboundMessage));
            return;
        }
        // check again if websocket ready in 10 milliseconds
        setTimeout(() => this.sendMessage(outboundMessage), 10);
    }

    /**
     * Get the existing instance of the game socket.
     */
    public static getInstance(): GameSocket | undefined {
        return GameSocket.instance;
    }

    /**
     * Create the underlying game socket.
     * @param socketUrl The URL for the websocket.
     */
    public static createInstance(socketUrl: string): GameSocket {
        // close existing socket if it's open
        GameSocket.instance?.websocket?.close();
        GameSocket.instance = new GameSocket(socketUrl);
        return GameSocket.instance;
    }

    /**
     * Destroy the underlying gamesocket by setting it to undefined.
     */
    public static destroyInstance() {
        GameSocket.instance?.websocket?.close();
        GameSocket.instance = undefined;
    }

    /**
     * Return the url used to establish the websocket connection.
     */
    public getSocketUrl(): string {
        return this.websocket.url;
    }

    /**
     * Get the lobby event.
     */
    public get onLobbyEvent(): ISimpleEvent<InboundMessage> {
        return this._onLobbyEvent.asEvent();
    }
}
