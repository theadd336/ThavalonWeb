import React, { useEffect, useState } from "react";
import { Container, ListGroup, Button } from "react-bootstrap";
import { GameSocket, InboundMessage, InboundMessageType, OutboundMessageType } from "../../utils/GameSocket";

/**
 * Component listing players currently in the lobby and a button to start the game.
 */
export function Lobby(props: any): JSX.Element {
    // useEffect handles componentDidMount and componentWillUnmount
    const [connection, setConnection] = useState<GameSocket | undefined>(undefined);
    const [playerList, setPlayerList] = useState<string[]>([]);

    function handleLobbyMessage(message: InboundMessage) {
        console.log(message.messageType);
        switch (message.messageType) {
            case InboundMessageType.PlayerList:
                setPlayerList(message.data as string[]);
                break;
        }
        return;
    }

    useEffect(() => {
        const newConnection = GameSocket.getInstance();
        newConnection.onLobbyEvent.subscribe(handleLobbyMessage);
        newConnection.sendMessage({ messageType: OutboundMessageType.GetPlayerList });
        setConnection(newConnection);

        return () => {
            if (connection !== undefined) {
                connection.onLobbyEvent.unsubscribe(handleLobbyMessage);
            }
        }
    }, []);

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