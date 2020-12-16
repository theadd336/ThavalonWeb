import React, { useEffect } from 'react';
import { GameSocket, InboundMessage } from '../utils/GameSocket';

export function Game(): JSX.Element {
    useEffect(() => {
        const gameSocket = GameSocket.getInstance();
        // on component did mount, subscribe to lobby events
        const unsubscribe = gameSocket.onLobbyEvent.subscribe((inboundMessage: InboundMessage) => {
            console.log(inboundMessage);
        });
        return () => {
            // on component unmount, unsubscribe from lobby events
            unsubscribe();
        }},
    // empty array should make it so above useEffect is only called on first render
    []);

    return (
        <div>
            <h1>Hello Game!</h1>
        </div>
    );
}