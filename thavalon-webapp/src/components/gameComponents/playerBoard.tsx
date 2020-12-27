import React, { useEffect, useState } from "react";
import { GameSocket, InboundMessage, InboundMessageType, OutboundMessageType } from "../../utils/GameSocket";
import { Vote, GameMessageType, GameMessage } from "./constants";
import { Spinner } from "react-bootstrap";

import "../../styles/gameStyles/playerBoard.scss";


interface PlayerCardProps {
    name: string
    toggleSelected: (name: string) => void,
    me?: boolean,
    tabbedOut?: boolean,
    isProposing?: boolean,
    isSelected?: boolean,
    vote?: Vote,
    declaredAs?: string
}

interface PlayerFocusChangeMessage {
    displayName: string,
    visibility: VisibilityState
}

export function PlayerBoard(): JSX.Element {
    useEffect(() => {
        const connection = GameSocket.getInstance();
        connection.onGameEvent.subscribe(handleMessage);
        connection.onLobbyEvent.subscribe(handleMessage);
        document.onvisibilitychange = () => toggleTabbedIndicator();
        return () => connection.onGameEvent.unsubscribe(handleMessage);
    }, [])

    // State for maintaining the player list.
    const [playerList, setPlayerList] = useState<string[]>([])
    // State maintaining selected players. These players are highlighted in green.
    const [selectedPlayers, setSelectedPlayers] = useState(new Set<string>());
    // State for maintaining players who are tabbed out. These players have a tab indicator.
    const [tabbedOutPlayers, setTabbedOutPlayers] = useState(new Set<string>());
    // I hate react sometimes
    const [test, setTest] = useState(0);

    function handleMessage(message: InboundMessage): void {
        switch (message.messageType) {
            case InboundMessageType.Snapshot:
                const snapshot = message.data as any
                handleGameMessage(snapshot.log[0] as GameMessage);
                break;
            case InboundMessageType.PlayerFocusChange:
                const { displayName, visibility } = message.data as PlayerFocusChangeMessage;
                playerFocusChanged(displayName, visibility);
                break;
            case InboundMessageType.GameMessage:
                handleGameMessage(message.data as GameMessage);
                break;
        }
    }

    function handleGameMessage(message: GameMessage): void {
        switch (message.messageType) {
            case GameMessageType.ProposalOrder:
                setPlayerList(message.data as string[]);
                break;
        }
    }

    function updateSelectedPlayers(name: string): void {
        updateSet(selectedPlayers, name, setSelectedPlayers);
    }

    function toggleTabbedIndicator(): void {
        const connection = GameSocket.getInstance();
        // visibilityState is either "hidden" or "visible." It can also be "prerender," but we don't talk about that.
        const message = { messageType: OutboundMessageType.PlayerFocusChange, data: document.visibilityState };
        connection.sendMessage(message);
    }

    function playerFocusChanged(player: string, visibility: VisibilityState) {
        console.log("Updating focus");
        console.log(`${ player } - ${ visibility }`);
        console.log(tabbedOutPlayers);
        console.log(test);
        if ((tabbedOutPlayers.has(player) && visibility === "visible")
            || (!tabbedOutPlayers.has(player) && visibility === "hidden")) {
            updateSet(tabbedOutPlayers, player, setTabbedOutPlayers);
            return;
        }
    }

    function updateSet<T>(setToUpdate: Set<T>, valueToToggle: T, reactSetter: React.Dispatch<React.SetStateAction<Set<T>>>): void {
        const tempSet = new Set<T>(setToUpdate.values());
        setTest(test + 1);
        if (!tempSet.delete(valueToToggle)) {
            tempSet.add(valueToToggle);
        }
        reactSetter(new Set<T>(tempSet.values()));
    }

    console.log("Prior to render");
    console.log(test);
    console.log(tabbedOutPlayers);
    const playerCards = playerList.map((playerName) => {
        return (
            <PlayerCard
                key={playerName}
                name={playerName}
                tabbedOut={tabbedOutPlayers.has(playerName)}
                isSelected={selectedPlayers.has(playerName)}
                toggleSelected={updateSelectedPlayers} />
        );
    });

    return (
        <div className="player-board">
            {playerCards}
        </div>
    );
}

function PlayerCard(props: PlayerCardProps): JSX.Element {
    return (
        <button
            className={`player-card ${ props.isSelected ? "player-selected" : "" }`}
            onClick={() => props.toggleSelected(props.name)}>
            <div>
                {props.tabbedOut &&
                    <Spinner
                        size="sm"
                        style={{ marginRight: "5px", marginBottom: "2px", position: "sticky" }}
                        variant="dark"
                        animation="border" />}
                {props.name}
            </div>
        </button>
    );
}