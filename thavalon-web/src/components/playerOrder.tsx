import { TabComponent } from "./tabComponents";
import { WebSocketProp } from "./communication";
import { IncomingMessage, IncomingMessageTypes } from "../Core/commConstants";
import { ListGroup } from "react-bootstrap";
import React from "react";

interface PlayerOrderMessage {
    playerOrder: string[];
}

interface PlayerOrderState {
    playerOrder: string[];
    activeProposer: string;
}

/**
 * Tab representing the order of proposals during the game.
 */
export class PlayerOrderTab extends TabComponent<PlayerOrderState> {
    /**
     * Initializes the player order tab.
     * @param props Props object containing the web socket manager.
     */
    constructor(props: WebSocketProp) {
        super(props);
        this.state = {playerOrder: [], activeProposer: ""};
    }

    /**
     * Handles any successful message received from the server.
     * @param _ Unused
     * @param message Incoming message from the server.
     */
    protected receiveSuccessfulMessage(_: object, message: IncomingMessage): void {
        if (message.type !== IncomingMessageTypes.PlayerOrder) {
            return;
        }
        const data = message.data as PlayerOrderMessage;
        const newState = {playerOrder: data.playerOrder};
        this.setState(newState);
    }

    /**
     * Renders the tab with the player order.
     */
    render(): JSX.Element {
        const playerOrderList = this.state.playerOrder.map((playerName) => {
            return (
                <ListGroup.Item 
                    as="li" 
                    key={playerName} 
                    active={this.state.activeProposer === playerName}>
                    {playerName}
                </ListGroup.Item>
            );
        });
        return (
            <ListGroup as="ol">
                {playerOrderList}
            </ListGroup>
        )
    }
}