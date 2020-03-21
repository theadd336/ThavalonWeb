import * as React from "react";
import { WebSocketProp, WebSocketManager } from "./communication"; 
import { MissingPropertyError } from "../Core/errors";
import { IncomingMessage, OutgoingMessage } from "../Core/commConstants";
/**
 * Abstract tab component class that sets up event handlers and cleans them up when the component will be destroyed.
 */
export abstract class TabComponent<S = {}> extends React.Component<WebSocketProp, S> {
    private _connection: WebSocketManager;
    /**
     * Initializes the class and sets up the connection.
     * @param props Props object with the WebSocketManager
     */
    constructor(props: WebSocketProp) {
        super(props);
        if (!(props.webSocket instanceof WebSocketManager)) {
            throw new MissingPropertyError("The WebSocketManager is missing.");
        }
        this._connection = props.webSocket;
    }

    /**
     * Sets up event handlers when the component mounts.
     */
    componentDidMount(): void {
        this._connection.onSuccessfulMessage.subscribe((sender, message) => {
            this.receiveSuccessfulMessage(sender, message);
        });
        
        this._connection.onErrorMessage.subscribe((sender, message) => {
            this.receiveErrorMessage(sender, message);
        });

        const message = this.sendMessageOnMount();
        if (message !== null) {
            this._connection.send(message);
        }
    }

    /**
     * Unsubscribes from events when the component is going to be destroyed.
     */
    componentWillUnmount(): void {
        this._connection.onSuccessfulMessage.unsubscribe((sender, message) => {
            this.receiveSuccessfulMessage(sender, message);
        });

        this._connection.onErrorMessage.unsubscribe((sender, message) => {
            this.receiveErrorMessage(sender, message);
        });
    }

    protected receiveSuccessfulMessage(sender: object, message: IncomingMessage): void {
    }

    protected receiveErrorMessage(sender: object, message: IncomingMessage): void {
    }

    protected sendMessage(message: OutgoingMessage): void {
        if (typeof message === "undefined") {
            return;
        }
        this._connection.send(message);
    }

    protected sendMessageOnMount(): OutgoingMessage | null {
        return null;
    }
}
