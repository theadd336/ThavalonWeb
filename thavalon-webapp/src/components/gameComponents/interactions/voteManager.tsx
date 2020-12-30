import React, { useEffect, useState } from "react";
import { ProgressBar } from "react-bootstrap";
import { GameSocket, InboundMessage, InboundMessageType } from "../../../utils/GameSocket";
import { InteractionProps, GameMessage, GameMessageType, Vote, GameActionType, VotingResultsMessage } from "../constants";
import { createSelectedPlayerTypesList, sendGameAction } from "../gameUtils";
import { PlayerCard } from "../playerCard";

interface VoteManagerProps extends InteractionProps {
    isMissionOne: boolean,
}

interface VoteButtonProps {
    isFirstMission: boolean,
    submitVote: (vote: Vote) => void,
}

export function VoteManager(props: VoteManagerProps): JSX.Element {
    const [hasVoted, setHasVoted] = useState(false);
    const [votesReceived, setVotesReceived] = useState(0);
    useEffect(() => {
        const connection = GameSocket.getInstance();
        connection.onGameEvent.subscribe(handleMessage);
        return () => connection.onGameEvent.unsubscribe(handleMessage);
    });

    function handleMessage(message: InboundMessage): void {
        if (message.messageType !== InboundMessageType.GameMessage) {
            return;
        }
        const gameMessage = message.data as GameMessage;
        if (gameMessage.messageType === GameMessageType.VoteRecieved) {
            setVotesReceived(votesReceived + 1);
        }
    }

    function submitVote(vote: Vote): void {
        sendGameAction(GameActionType.Vote, { upvote: Boolean(vote) });
        setHasVoted(true);
    }

    const playerCards = props.playerList.map((playerName) => {
        const selectedTypes = createSelectedPlayerTypesList(playerName, props.primarySelectedPlayers, props.secondarySelectedPlayers);
        return <PlayerCard
            key={playerName}
            name={playerName}
            tabbedOut={props.tabbedOutPlayers.has(playerName)}
            selectedTypes={selectedTypes}
            enabled={false} />
    });

    const numPlayers = props.playerList.length;
    return (
        <>
            {playerCards}
            <div className="vote-manager">
                {hasVoted ?
                    <ProgressBar now={votesReceived * 100 / numPlayers} label={`${ votesReceived } / ${ numPlayers }`} />
                    :
                    <VoteButtons
                        isFirstMission={props.isMissionOne}
                        submitVote={submitVote} />}
            </div>
        </>
    );
}

function VoteButtons(props: VoteButtonProps): JSX.Element {
    return (
        <>
            <button
                className="vote-button green"
                onClick={() => props.submitVote(Vote.Upvote)}>
                {props.isFirstMission ? "Send Green" : "Accept"}
            </button>
            <button
                className="vote-button red"
                onClick={() => props.submitVote(Vote.Downvote)}>
                {props.isFirstMission ? "Send Red" : "Decline"}
            </button>
        </>
    );
}