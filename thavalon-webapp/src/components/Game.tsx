import React from 'react';
import { GameSocket, InboundMessage, OutboundMessageType } from '../utils/GameSocket';

export function Game(): JSX.Element {
    const gameSocket = GameSocket.getInstance();
    gameSocket.onLobbyEvent.subscribe((inboundMessage: InboundMessage) => {
        console.log(inboundMessage);
    });
    gameSocket.sendMessage({
        messageType: OutboundMessageType.Ping,
    });

    return (
        <div>
            <h1>Hello Game!</h1>
        </div>
    );
}