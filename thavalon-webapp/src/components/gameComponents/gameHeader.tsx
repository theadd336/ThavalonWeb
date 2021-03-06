import React, { useEffect, useState } from "react";
import { GameSocket, InboundMessage, InboundMessageType } from "../../utils/GameSocket";
import { GameMessage, GameMessageType, Snapshot, NextProposalMessage } from "./constants";
import { GamePhase, mapMessageToGamePhase } from "./gameUtils";

import "../../styles/gameStyles/gameGlobals.scss";

/**
 * An enum of game phase header titles.
 */
enum GamePhaseHeader {
    ProposalOther = "Waiting for Proposal",
    ProposalSelf = "Propose",
    Vote = "Vote",
    VoteResults = "Voting Results",
    Mission = "Questing",
    Assassination = "Assassination"
}

/**
 * Creates the gamephase header and force count component.
 */
export function GameHeader(): JSX.Element {
    // State for the current phase of the game
    const [gamePhaseHeader, setGamePhaseHeader] = useState(GamePhaseHeader.ProposalOther);
    // State for the number of turns until force. Initialized to 1 to prevent any force-specific actions from occurring.
    const [turnsUntilForce, setTurnsUntilForce] = useState(1);
    // State to determine the name of this player.
    const [me, setMe] = useState("");

    useEffect(() => {
        const connection = GameSocket.getInstance();
        connection.onLobbyEvent.subscribe(handleMessage);
        connection.onGameEvent.subscribe(handleMessage);
        return () => {
            connection.onLobbyEvent.unsubscribe(handleMessage);
            connection.onGameEvent.unsubscribe(handleMessage);
        }
    });

    /**
     * Handles any message received from the server.
     * @param message An inbound message from the server
     */
    function handleMessage(message: InboundMessage): void {
        switch (message.messageType) {
            // If it's a snapshot, check the last message to get the current phase.
            case InboundMessageType.Snapshot:
                const snapshot = message.data as Snapshot;
                setMe(snapshot.me);
                const lastLogIndex = snapshot.log.length - 1;
                const lastMessage = snapshot.log[lastLogIndex];
                mapMessageToCaption(lastMessage);
                // The last message will always give us the state, but to figure out
                // the number of proposals remaining, we need a NextProposal message.
                // Loop backwards over the messages until we find one.
                for (let i = lastLogIndex; i >= 0; i--) {
                    const logMessage = snapshot.log[i];
                    if (logMessage.messageType === GameMessageType.NextProposal) {
                        const proposalMessage = logMessage.data as NextProposalMessage;
                        setTurnsUntilForce(proposalMessage.max_proposals - proposalMessage.proposals_made);
                        break;
                    }
                }
                break;
            // If it's a normal game message, just pass in the data to determine the phase.
            case InboundMessageType.GameMessage:
                mapMessageToCaption(message.data as GameMessage);
                break;
        }
    }

    /**
     * Given a message, determines which phase of the game the player is currently in.
     * @param message A GameMessage from the server
     */
    function mapMessageToCaption(message: GameMessage): void {
        const gamePhase = mapMessageToGamePhase(message.messageType);
        switch (gamePhase) {
            case GamePhase.Assassination:
                setGamePhaseHeader(GamePhaseHeader.Assassination);
                break;
            case GamePhase.Vote:
                setGamePhaseHeader(GamePhaseHeader.Vote);
                break;
            case GamePhase.Proposal:
                // Technically, proposal made is still in the proposal phase, but
                // we shouldn't make any state updates on it, since we're about
                // to be in voting.
                if (message.messageType !== GameMessageType.NextProposal) {
                    return;
                }
                // If it's the next proposal, we need to see if it's us proposing or not.
                // Also, update the force counter here.
                const proposalMessage = message.data as NextProposalMessage;
                setTurnsUntilForce(proposalMessage.max_proposals - proposalMessage.proposals_made);
                if (proposalMessage.proposer === me) {
                    setGamePhaseHeader(GamePhaseHeader.ProposalSelf);
                } else {
                    setGamePhaseHeader(GamePhaseHeader.ProposalOther);
                }
                break;
            case GamePhase.Mission:
                setGamePhaseHeader(GamePhaseHeader.Mission);
                break;
        }
    }

    return (
        <div>
            <h1 className="game-section-header">
                Game: {gamePhaseHeader}
            </h1>
            <h3 className="game-section-subheader">
                Turns until force: {turnsUntilForce}
            </h3>
        </div>
    );
}