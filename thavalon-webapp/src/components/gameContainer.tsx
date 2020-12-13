import React, { useEffect, useState } from "react";
import { } from "react-router";
import { Lobby } from "./gameComponents/lobby";
import { GameSocket } from "../utils/GameSocket";

interface GameContainerProps {
    location: {
        state: {
            socketUrl: string,
            friendCode: string
        }
    }
}

export function GameContainer(props: GameContainerProps): JSX.Element {
    const [connection, setConnection] = useState<GameSocket | undefined>(undefined);
    if (connection === undefined || connection.socketUrl !== props.location.state.socketUrl) {
        const temp = GameSocket.createInstance(props.location.state.socketUrl);
        setConnection(temp);
    }
        return <Lobby friendCode={props.location.state.friendCode} />;
}