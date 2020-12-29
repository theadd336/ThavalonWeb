import React, { useEffect, useState } from "react";
import { GameSocket, InboundMessage, InboundMessageType, OutboundMessageType } from "../../utils/GameSocket";
import { Vote, GameMessageType, GameMessage, Snapshot } from "./constants";
import { Spinner } from "react-bootstrap";

import "../../styles/gameStyles/playerBoard.scss";


/**
 * Props object for the PlayerCard component. Some of these aren't used yet,
 * pending future development.
 */
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

/**
 * Message for the tabbed out indicator message.
 */
interface PlayerFocusChangeMessage {
    displayName: string,
    isTabbedOut: boolean
}

/**
 * Board containing the player list and all related functions. This is a container
 * for all PlayerCards and has handlers for game and lobby events.
 */
export function PlayerBoard(): JSX.Element {
    // On mount, set up event handlers for the game socket and the DOM.
    // On unmount, clean up event handlers.
    useEffect(() => {
        const connection = GameSocket.getInstance();
        connection.onGameEvent.subscribe(handleMessage);
        connection.onLobbyEvent.subscribe(handleMessage);
        document.onvisibilitychange = () => sendPlayerVisibilityChange();
        return () => {
            connection.onGameEvent.unsubscribe(handleMessage);
            document.onvisibilitychange = null;
        }
    }, [])

    // State for maintaining the player list.
    const [playerList, setPlayerList] = useState<string[]>([])
    // State maintaining selected players. These players are highlighted in green.
    const [selectedPlayers, setSelectedPlayers] = useState(new Set<string>());
    // State for maintaining players who are tabbed out. These players have a tab indicator.
    const [tabbedOutPlayers, setTabbedOutPlayers] = useState(new Set<string>());

    /**
     * Generic message handler for all messages from the server
     * @param message The InboundMessage from the server.
     */
    function handleMessage(message: InboundMessage): void {
        switch (message.messageType) {
            case InboundMessageType.Snapshot:
                const snapshot = message.data as Snapshot
                handleGameMessage(snapshot.log[0]);
                break;
            case InboundMessageType.PlayerFocusChange:
                const { displayName, isTabbedOut } = message.data as PlayerFocusChangeMessage;
                playerFocusChanged(displayName, isTabbedOut);
                break;
            case InboundMessageType.GameMessage:
                handleGameMessage(message.data as GameMessage);
                break;
        }
    }

    /**
     * GameMessage specific message handler. This is needed because the GameMessage
     * has internal types to parse.
     * @param message The GameMessage from the server
     */
    function handleGameMessage(message: GameMessage): void {
        switch (message.messageType) {
            case GameMessageType.ProposalOrder:
                setPlayerList(message.data as string[]);
                break;
        }
    }

    /**
     * Updates the selected player list with a new name.
     * @param name The player name to select.
     */
    function updateSelectedPlayers(name: string): void {
        updateSet(selectedPlayers, name, setSelectedPlayers);
    }

    /**
     * Sends a message to the server that the player is tabbed in or out.
     */
    function sendPlayerVisibilityChange(): void {
        const connection = GameSocket.getInstance();
        const isTabbedOut = document.visibilityState === "hidden" ? true : false;
        const message = { messageType: OutboundMessageType.PlayerFocusChange, data: isTabbedOut };
        connection.sendMessage(message);
    }

    /**
     * Updates the tabbed out player list with a new player and visibility
     * @param player The player whose focus has changed
     * @param visibility The new visibility for that player
     */
    function playerFocusChanged(player: string, isTabbedOut: boolean): void {
        const tempSet = new Set(tabbedOutPlayers.values());
        if (isTabbedOut) {
            tempSet.add(player);
        } else {
            tempSet.delete(player);
        }
        setTabbedOutPlayers(tempSet);
    }

    /**
     * Helper function to update a set.
     * @param setToUpdate The set to update
     * @param valueToToggle The value in the set to toggle
     * @param reactSetter The React setter for the state to update
     */
    function updateSet<T>(setToUpdate: Set<T>, valueToToggle: T, reactSetter: React.Dispatch<React.SetStateAction<Set<T>>>): void {
        // Use tempSet since react stateful variables must never be modified directly
        const tempSet = new Set<T>(setToUpdate.values());
        if (!tempSet.delete(valueToToggle)) {
            tempSet.add(valueToToggle);
        }
        reactSetter(tempSet);
    }

    // Create the player cards with the state we have.
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

/**
 * React component representing an interactive player button. This button doesn't
 * directly communicate with the server but handles all styling of relevant icons.
 * @param props The props for the player card
 */
function PlayerCard(props: PlayerCardProps): JSX.Element {
    return (
        <button
            className={`player-card ${ props.isSelected ? "player-selected" : "" }`}
            onClick={() => props.toggleSelected(props.name)}>
            <div>
                {props.tabbedOut &&
                    <Spinner
                        className="tabbed-out-indicator"
                        size="sm"
                        variant="dark"
                        animation="border" />}
                {props.name}
            </div>
        </button>
    );
}