import React from 'react';
import { GameSocket } from '../utils/GameSocket';

export function Game(): JSX.Element {
    const gameSocket = GameSocket.getInstance();
    gameSocket.sendPing();

    return (
        <div>
            <h1>Hello Game!</h1>
        </div>
    );
}