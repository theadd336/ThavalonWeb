import React, { useEffect, useState } from "react";
import { Lobby } from "./gameComponents/lobby";
import { GameRoot } from "./gameComponents/gameRoot";
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
    Loading = "Loading",
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
    // State to maintain the current status of the lobby.
    const [lobbyState, setLobbyState] = useState(LobbyState.Loading);

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
        // The game container maintains the connection. If one exists, destroy it.
        if (GameSocket.getInstance()) {
            GameSocket.destroyInstance();
        }
        const connection = GameSocket.createInstance(props.location.state.socketUrl);
        connection.onLobbyEvent.subscribe(receiveLobbyMessage);
        connection.sendMessage({ messageType: OutboundMessageType.GetLobbyState });
        return () => {
            const connection = GameSocket.getInstance();
            connection?.onLobbyEvent.unsubscribe(receiveLobbyMessage);
            GameSocket.destroyInstance();
        }
    }, []);

    return (
        <>
            {lobbyState === LobbyState.Loading && <h1>Loading</h1>}
            {lobbyState === LobbyState.Lobby && <Lobby friendCode={props.location.state.friendCode} />}
            {lobbyState === LobbyState.Game && <GameRoot />}
        </>
    );

}