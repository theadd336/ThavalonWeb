import React, { useEffect, useState } from "react";
import { Lobby } from "./gameComponents/lobby";
import { GameSocket, OutboundMessageType, InboundMessage, InboundMessageType } from "../utils/GameSocket";

/**
 * Props interface for the GameContainer. This structure matches the object
 * passed by <Route /> to extract the required state information.
 */
interface GameContainerProps {
    location: {
        state: {
            socketUrl: string,
            friendCode: string
        }
    }
}

/**
 * Enum of lobby states to determine what is shown to the user.
 */
enum LobbyState {
    Lobby = "Lobby",
    Game = "Game",
}

/**
 * Interface for the incoming lobby state message.
 */
interface LobbyStateResponse {
    state: LobbyState
}

/**
 * Contains all of the game related components and sets up low level connection
 * infrastructure.
 * @param props Props object for the GameContainer
 */
export function GameContainer(props: GameContainerProps): JSX.Element {
    // State to maintain a connection instance so we don't have to getInstance every time.
    const [connection, setConnection] = useState<GameSocket | undefined>(undefined);
    // State to maintain the current status of hte lobby.
    const [lobbyState, setLobbyState] = useState(LobbyState.Lobby);

    /**
     * Handles an incoming lobby message. If the message is a state change,
     * the GameContainer will update states appropriately.
     * @param message An incoming message from the server.
     */
    function receiveLobbyMessage(message: InboundMessage): void {
        if (message.messageType === InboundMessageType.LobbyState) {
            const data = message.data as LobbyStateResponse;
            setLobbyState(data.state);
        }
    }

    // useEffect return === componentWillUnmount in class React. Use componentWillUnmount
    // to remove our event handler.
    useEffect(() => {
        return () => {
            connection?.onLobbyEvent.unsubscribe(receiveLobbyMessage);
            GameSocket.destroyInstance();
        }
    }, []);

    // If we don't have a connection or the state URL changed, update with a new connection.
    if (connection === undefined ||
        connection.getSocketUrl() !== props.location.state.socketUrl) {
        const newConnection = GameSocket.createInstance(props.location.state.socketUrl);
        newConnection.onLobbyEvent.subscribe(receiveLobbyMessage);
        newConnection.sendMessage({ messageType: OutboundMessageType.GetLobbyState });
        setConnection(newConnection);
    }

    return (
        <>
            {lobbyState === LobbyState.Lobby && <Lobby friendCode={props.location.state.friendCode} />}
            {lobbyState === LobbyState.Game && <h1>Not Implemented (yet..)</h1>}
        </>
    );

}