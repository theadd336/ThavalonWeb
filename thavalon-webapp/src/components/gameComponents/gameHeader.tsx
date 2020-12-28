import React, { useEffect, useState } from "react";
import { GameSocket, InboundMessage, InboundMessageType } from "../../utils/GameSocket";
import { GameMessage, GameMessageType, Snapshot, NextProposalMessage } from "./constants";

import "../../styles/gameStyles/gameGlobals.scss";

enum GamePhaseHeader {
    ProposalOther = "Waiting for Proposal",
    ProposalSelf = "Propose",
    Vote = "Vote",
    VoteResults = "Voting Results",
    Mission = "Questing",
    Assassination = "Assassination"
}

/**
 * 
 */
export function GameHeader(): JSX.Element {
    const [gamePhaseHeader, setGamePhaseHeader] = useState(GamePhaseHeader.ProposalOther);
    const [turnsUntilForce, setTurnsUntilForce] = useState(1);
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

    function handleMessage(message: InboundMessage): void {
        switch (message.messageType) {
            case InboundMessageType.Snapshot:
                const snapshot = message.data as Snapshot;
                setMe(snapshot.me);
                const lastLogIndex = snapshot.log.length - 1;
                mapMessageToState(snapshot.log[lastLogIndex]);
                break;
            case InboundMessageType.GameMessage:
                mapMessageToState(message.data as GameMessage);
                break;
        }
    }

    function mapMessageToState(message: GameMessage): void {
        switch (message.messageType) {
            case GameMessageType.BeginAssassination:
                setGamePhaseHeader(GamePhaseHeader.Assassination);
                break;
            case GameMessageType.CommenceVoting:
                setGamePhaseHeader(GamePhaseHeader.Vote);
                break;
            case GameMessageType.NextProposal:
                const proposalMessage = message.data as NextProposalMessage;
                setTurnsUntilForce(proposalMessage.maxProposals - proposalMessage.proposalsMade);
                if (proposalMessage.proposer === me) {
                    setGamePhaseHeader(GamePhaseHeader.ProposalSelf);
                } else {
                    setGamePhaseHeader(GamePhaseHeader.ProposalOther);
                }
                break;
            case GameMessageType.MissionGoing:
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