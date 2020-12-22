import React, { useEffect, useState } from "react";
import { GameSocket, InboundMessage, InboundMessageType, OutboundMessageType } from "../../utils/GameSocket";
import "../../styles/gameStyles/roleInformation.scss";

interface RoleInfo {
    abilities: string,
    assassinatable: boolean,
    description: string,
    isAssassin: boolean,
    otherInfo: string,
    priorityTarget: string | undefined,
    role: string,
    seenPlayers: string[],
    team: string,
    teamMembers: string[],
}

interface Snapshot {
    roleInfo: RoleInfo,
    missions: string[],
    log: string[],
}

export function RoleInformation(): JSX.Element {
    const [roleInfoElement, setRoleInfoElement] = useState<JSX.Element>(<h1>Loading Role Info...</h1>);

    /**
     * Creates the JSX Element for representing the role information
     * section of a game page.
     * @param roleInfo The role info received in a snapshot message.
     */
    function createRoleInfoElement(roleInfo: RoleInfo): JSX.Element {
        return (
            <div id="playerInfo">
                <h1 className="gameSectionHeader">Player Info</h1>
                You are <span className={roleInfo.team.toLowerCase()}>{roleInfo.role}: {roleInfo.team.toUpperCase()}</span>
                <br />
                {roleInfo.description && <>
                    <span className="roleDescription">{roleInfo.description}</span>
                    <br />
                </>}

                {roleInfo.abilities && <>
                    <span className="abilities">{roleInfo.abilities}</span>
                    <br />
                </>}

                {roleInfo.seenPlayers.length > 0 && <>
                    <span className="seenPlayers">You see {roleInfo.seenPlayers.join(", ")}</span>
                    <br />
                </>}

                {roleInfo.team == "Evil" && roleInfo.teamMembers.length > 0 && <>
                    <span className="teamMembers">You see {roleInfo.teamMembers.join(", ")} as evil</span>
                    <br />
                </>}

                {roleInfo.otherInfo && <>
                    <span className="otherInfo">{roleInfo.otherInfo}</span>
                    <br />
                </>}

                {roleInfo.assassinatable && <>
                    <span className="assassinatable">You are assassinatable!</span>
                    <br />
                </>}

                {roleInfo.isAssassin && <>
                    <span className="assassin">You are the assassin!</span>
                    <br />
                    {roleInfo.priorityTarget !== "None" && <span className="priorityTarget">{roleInfo.priorityTarget} is the priority target!</span>}
                    {roleInfo.priorityTarget === "None" && <span className="priorityTarget">There is no priority target.</span>}
                    <br />
                </>}
            </div>
        )
    }

    /**
     * Handles any lobby messages that come from the server. If the message type
     * is a PlayerList change, the playerList is updated accordingly.
     * @param message An incoming message from the server
     */
    function handleGameMessage(message: InboundMessage): void {
        switch (message.messageType) {
            case InboundMessageType.Snapshot: {
                const snapshot = message.data as Snapshot;
                console.log("SETTING ROLE INFO ELEMENT!");
                setRoleInfoElement(createRoleInfoElement(snapshot.roleInfo));
                break;
            }
            default: {
                console.log("Received unsupported message type: " + message.messageType);
                break;
            }
        }
    }

    // useEffect handles componentDidMount and componentWillUnmount steps.
    useEffect(() => {
        // On mount, get the connection instance and set up event handlers.
        // Then, get the player list.
        const connection = GameSocket.getInstance();
        connection?.onGameEvent.subscribe(handleGameMessage);
        console.log("SET UP MESSAGE!");
        connection?.sendMessage({ messageType: OutboundMessageType.GetSnapshot });

        // On unmount, unsubscribe our event handlers.
        return () => {
            const connection = GameSocket.getInstance();
            connection?.onGameEvent.unsubscribe(handleGameMessage);
        }
    }, []);
    
    return roleInfoElement;
}