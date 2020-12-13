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

enum LobbyState {
    Lobby = "Lobby",
    Game = "Game",
}

interface LobbyStateResponse {
    state: LobbyState
}

/**
 * Contains all of the game related components and sets up low level connection
 * infrastructure.
 * @param props Props object for the GameContainer
 */
export function GameContainer(props: GameContainerProps): JSX.Element {
    const [connection, setConnection] = useState<GameSocket | undefined>(undefined);
    const [lobbyState, setLobbyState] = useState(LobbyState.Lobby);

    function receiveLobbyMessage(message: InboundMessage): void {
        if (message.messageType === InboundMessageType.LobbyState) {
            const data = message.data as LobbyStateResponse;
            setLobbyState(data.state);
        }
    }

    useEffect(() => {
        return () => {
            connection?.onLobbyEvent.unsubscribe(receiveLobbyMessage);
            GameSocket.destroyInstance();
        }
    }, []);

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