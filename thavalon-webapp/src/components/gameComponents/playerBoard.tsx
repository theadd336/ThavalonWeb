import React, { useEffect, useState } from "react";
import { GameSocket, InboundMessage, InboundMessageType } from "../../utils/GameSocket";
import { Vote } from "./constants";

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

export function PlayerBoard(): JSX.Element {
    useEffect(() => {
        const connection = GameSocket.getInstance();
        
    }, [])

    // State for maintaining the player list.
    const [playerList, setPlayerList] = useState(Array<string>())
    // State maintaining selected players. These players are highlighted in green.
    const [selectedPlayers, setSelectedPlayers] = useState(new Set<string>());

    function handleMessage(message: InboundMessage): void {
        switch (message.messageType) {
            case InboundMessageType.Snapshot:
                break;
            case InboundMessageType.
        }
    }
    function updateSelectedPlayers(name: string): void {
        if (!selectedPlayers.delete(name)) {
            selectedPlayers.add(name);
            setSelectedPlayers(selectedPlayers);
        }
    }

    const playerCards = playerList.map((playerName) => {
        return (
            <PlayerCard
                name={playerName}
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
        <div className={`player-card ${ props.isSelected ? "player-selected" : "" }`}>
            {props.name}
        </div>
    );
}