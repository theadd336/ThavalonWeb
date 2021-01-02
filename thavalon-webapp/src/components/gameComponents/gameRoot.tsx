import React, { useEffect } from "react";
import { GameSocket, ConnectionError, OutboundMessageType } from "../../utils/GameSocket";
import { RoleInformation } from "./roleInformation";
import { MissionResults } from "./missionResults";
import { PlayerBoard } from "./playerBoard";
import { GameHeader } from "./gameHeader";

import "../../styles/gameStyles/gameGlobals.scss";
import { Notifications } from "./notifications";

export function GameRoot(): JSX.Element {
    // ComponentDidMount runs after all children render, so we can send the 
    // request for all game snapshots here.
    useEffect(() => {
        // This is a hack. We should support a light and dark mode theme for this
        // at the app level.
        document.body.classList.add("game-background-color");
        const connection = GameSocket.getInstance();

        // The connection should *always* exist at this point.
        if (connection === undefined) {
            throw new ConnectionError();
        }
        connection.sendMessage({ messageType: OutboundMessageType.GetSnapshot });

        // Destroy the existing connection when this unmounts.
        return () => {
            GameSocket.destroyInstance();
            document.body.classList.remove("game-background-color");
        }
    }, []);

    return (
        <div className="game-root-container">
            <div className="col-left">
                <div className="row-top">
                    <RoleInformation />
                </div>
                <div className="row-bottom">
                    <MissionResults />
                </div>
            </div>
            <div className="col-center">
                <GameHeader />
                <PlayerBoard />
            </div>
            <div className="col-right">
                <Notifications />
            </div>
        </div>
    );
}