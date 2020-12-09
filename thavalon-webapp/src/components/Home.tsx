import React from 'react';
import { AccountManager, HttpResponse } from '../utils/AccountManager';
import { GameSocket } from '../utils/GameSocket';

export function Home(): JSX.Element {
    const accountManager = AccountManager.getInstance();

    function makeGame() {
        accountManager.createGame().then((response: HttpResponse) => {
            const name = "test";
            const friendCode = response.message;
            console.log(friendCode);
            accountManager.joinGame(friendCode, name).then((response: HttpResponse) => {
                console.log(response.message);
                const gameSocket = GameSocket.getInstance(response.message);
                console.log(gameSocket);
                gameSocket.sendPing();
            });
        });
    }

    return (
        <div>
            <h1>Hello World!</h1>
        </div>
    );
}