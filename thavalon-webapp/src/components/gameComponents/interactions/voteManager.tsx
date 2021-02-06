import React, { useEffect, useState } from "react";
import { ProgressBar } from "react-bootstrap";
import { GameSocket, InboundMessage, InboundMessageType } from "../../../utils/GameSocket";
import { InteractionProps, GameMessage, GameMessageType, Vote, GameActionType, VotingResultsMessage, Role } from "../constants";
import { createSelectedPlayerTypesList, sendGameAction } from "../gameUtils";
import { PlayerCard } from "../playerCard";

import "../../../styles/gameStyles/interactionStyles/voteManager.scss";

/**
 * Props object for the VoteManager.
 */
interface VoteManagerProps extends InteractionProps {
    isMissionOne: boolean,
}

/**
 * Props object for the voting buttons
 */
interface VoteButtonProps {
    isFirstMission: boolean,
    submitVote: (vote: Vote) => void,
    submitObscure: () => void,
    showObscure: boolean,
}

/**
 * Component handling all voting related interactions.
 * Since votes need to be shared, VotingResults are handled by the playerBoard.
 * @param props The required properties for the VoteManager
 */
export function VoteManager(props: VoteManagerProps): JSX.Element {
    // State for checking if the player has voted or not.
    const [hasVoted, setHasVoted] = useState(false);
    // State for counting the number of votes received.
    const [votesReceived, setVotesReceived] = useState(0);
    // State for maintaining if maeve can obscure. Note: the way
    // this is handled isn't great. We should have the server tell us that Maeve
    // can obscure here instead of deriving this state, since Maeve can only obscure twice.
    const [showObscure, setShowObscure] = useState(props.role === Role.Maeve && !props.isMissionOne);

    useEffect(() => {
        const connection = GameSocket.getInstance();
        connection.onGameEvent.subscribe(handleMessage);
        return () => connection.onGameEvent.unsubscribe(handleMessage);
    });

    /**
     * Handles any incoming message from the server
     * @param message A message from the server
     */
    function handleMessage(message: InboundMessage): void {
        if (message.messageType !== InboundMessageType.GameMessage) {
            return;
        }
        const gameMessage = message.data as GameMessage;
        if (gameMessage.messageType === GameMessageType.VoteRecieved) {
            setVotesReceived(votesReceived + 1);
        }
    }

    /**
     * Submits a vote to the server and updates the component to show
     * the in-progress bar.
     * @param vote The vote to submit
     */
    function submitVote(vote: Vote): void {
        sendGameAction(GameActionType.Vote, { upvote: Boolean(vote) });
        setHasVoted(true);
    }

    /**
     * Submits an obscure to the server and removes the Obscure button.
     */
    function submitObscure(): void {
        sendGameAction(GameActionType.Obscure);
        setShowObscure(false);
    }

    // Create the player cards here.
    const playerCards = props.playerList.map((playerName) => {
        const selectedTypes = createSelectedPlayerTypesList(playerName, props.primarySelectedPlayers, props.secondarySelectedPlayers);
        return <PlayerCard
            key={playerName}
            name={playerName}
            tabbedOut={props.tabbedOutPlayers.has(playerName)}
            selectedTypes={selectedTypes}
            enabled={false}
            declaredAs={props.declarationMap.get(playerName)}
        />
    });

    const numPlayers = props.playerList.length;
    return (
        <>
            {playerCards}
            <div className="interaction-manager">
                {hasVoted ?
                    <ProgressBar style={{ minWidth: "200px" }}
                        now={votesReceived * 100 / numPlayers} label={`${ votesReceived } / ${ numPlayers }`} />
                    :
                    <VoteButtons
                        submitObscure={submitObscure}
                        showObscure={showObscure}
                        isFirstMission={props.isMissionOne}
                        submitVote={submitVote} />}
            </div>
        </>
    );
}

/**
 * Component that manages the voting buttons.
 * @param props Required properties for the voting buttons
 */
function VoteButtons(props: VoteButtonProps): JSX.Element {
    return (
        <>
            <button
                className="vote-button-green"
                onClick={() => props.submitVote(Vote.Upvote)}>
                {props.isFirstMission ? "Send Green" : "Accept"}
            </button>
            <button
                className={props.isFirstMission ? "vote-button-blue" : "vote-button-red"}
                onClick={() => props.submitVote(Vote.Downvote)}>
                {props.isFirstMission ? "Send Blue" : "Decline"}
            </button>
            {props.showObscure &&
                <button
                    className="obscure-button"
                    onClick={() => props.submitObscure()}>
                    Obscure
                </button>
            }
        </>
    );
}