import React, { useCallback, useEffect, useState } from "react";
import { GameSocket, InboundMessage, InboundMessageType, OutboundMessageType } from "../../utils/GameSocket";
import { GameActionType, RoleInfo, Snapshot, GameMessage, GameMessageType, Role } from "./constants";
import "../../styles/gameStyles/roleInformation.scss";
import { sendGameAction } from "./gameUtils";

/**
 * The role info of the player in the game.
 */
export function RoleInformation(): JSX.Element {
    const [roleInfo, setRoleInfo] = useState<RoleInfo | undefined>(undefined);
    const [showDeclareButton, setShowDeclareButton] = useState(false);

    /**
     * Handles any lobby messages that come from the server. If the message type
     * is a PlayerList change, the playerList is updated accordingly.
     * @param message An incoming message from the server.
     */
    function handleMessage(message: InboundMessage): void {
        switch (message.messageType) {
            case InboundMessageType.Snapshot: {
                const snapshot = message.data as Snapshot;
                setRoleInfo(snapshot.roleInfo);
                break;
            }
            case InboundMessageType.GameMessage: {
                // For now, only Arthur has a declare button here.
                if (roleInfo?.role !== Role.Arthur) {
                    return;
                }
                const gameMessage = message.data as GameMessage;
                if (gameMessage.messageType === GameMessageType.ArthurCanDeclare) {
                    setShowDeclareButton(true);
                } else if (gameMessage.messageType === GameMessageType.ArthurCannotDeclare) {
                    setShowDeclareButton(false);
                }
                break;
            }
        }
    }

    /**
     * Wrapper function that sends a "Declare" action for Arthur.
     */
    function submitArthurDeclaration(): void {
        sendGameAction(GameActionType.Declare);
        setShowDeclareButton(false);
    }

    // useEffect handles componentDidMount and componentWillUnmount steps.
    useEffect(() => {
        // On mount, get the connection instance and set up event handlers.
        // Then, get the player list.
        const connection = GameSocket.getInstance();
        connection.onGameEvent.subscribe(handleMessage);

        // On unmount, unsubscribe our event handlers.
        return () => {
            connection.onGameEvent.unsubscribe(handleMessage);
        }
    }, [handleMessage]);

    if (roleInfo === undefined) {
        return <></>
    }

    return <div id="playerInfo">
        <h1 className="game-section-header">Player Info</h1>
        You are <span className={roleInfo.team.toLowerCase()}>{roleInfo.role}: {roleInfo.team.toUpperCase()}</span>
        <ul>
            {roleInfo.description &&
                <li><span className="role-description">{roleInfo.description}</span></li>
            }

            {roleInfo.abilities &&
                <li><span className="abilities">{roleInfo.abilities}</span></li>
            }

            {roleInfo.seenPlayers.length > 0 &&
                <li><span className="seen-players">You see {roleInfo.seenPlayers.join(", ")}</span></li>
            }

            {roleInfo.team === "Evil" && roleInfo.teamMembers.length > 0 &&
                <li><span className="team-members">You see {roleInfo.teamMembers.join(", ")} as evil</span></li>
            }

            {roleInfo.otherInfo &&
                <li><span className="other-info">{roleInfo.otherInfo}</span></li>
            }

            {roleInfo.assassinatable &&
                <li><span className="assassinatable">You are assassinatable!</span></li>
            }

            {roleInfo.isAssassin &&
                <li><span className="assassin">You are the assassin! </span>
                    {roleInfo.priorityTarget !== "None" && <span className="priority-target">{roleInfo.priorityTarget} is the priority target!</span>}
                    {roleInfo.priorityTarget === "None" && <span className="priority-target">There is no priority target.</span>}</li>
            }
        </ul>
        <div className="center-content">
            {showDeclareButton &&
                <button
                    className="declare-button-good"
                    onClick={() => submitArthurDeclaration()}>

                    Declare as Arthur
            </button>}
        </div>
    </div>

}