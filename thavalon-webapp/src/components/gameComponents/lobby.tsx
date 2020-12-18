import React, { useEffect, useState } from "react";
import { Container, ListGroup, Button } from "react-bootstrap";
import { GameSocket, InboundMessage, InboundMessageType, OutboundMessageType } from "../../utils/GameSocket";

/**
 * Interface for the Lobby props object.
 */
interface LobbyProps {
    friendCode: string
}

/**
 * Component listing players currently in the lobby and a button to start the game.
 */
export function Lobby(props: LobbyProps): JSX.Element {
    // State for maintaining the player list.
    const [playerList, setPlayerList] = useState<string[]>([]);

    /**
     * Handles any lobby messages that come from the server. If the message type
     * is a PlayerList change, the playerList is updated accordingly.
     * @param message An incoming message from the server
     */
    function handleLobbyMessage(message: InboundMessage): void {
        if (message.messageType === InboundMessageType.PlayerList) {
            setPlayerList(message.data as string[]);
        }
    }

    // useEffect handles componentDidMount and componentWillUnmount steps.
    useEffect(() => {
        // On mount, get the connection instance and set up event handlers.
        // Then, get the player list.
        const connection = GameSocket.getInstance();
        connection?.onLobbyEvent.subscribe(handleLobbyMessage);
        connection?.sendMessage({ messageType: OutboundMessageType.GetPlayerList });

        // On unmount, unsubscribe our event handlers.
        return () => {
            const connection = GameSocket.getInstance();
            connection?.onLobbyEvent.unsubscribe(handleLobbyMessage);
        }
    }, []);

    const connection = GameSocket.getInstance();
    // Create the player ListGroup items with each player name.
    const players = playerList.map((player) =>
        <ListGroup.Item key={player}>{player}</ListGroup.Item>
    );

    return (
        <Container>
            <h1>Friend Code: {props.friendCode}</h1>
            <ListGroup variant="flush">
                {players}
            </ListGroup>
            <Button
                variant="primary"
                onClick={() => connection?.sendMessage({ messageType: OutboundMessageType.StartGame })}>
                Start Game
            </Button>
        </Container>
    );
}