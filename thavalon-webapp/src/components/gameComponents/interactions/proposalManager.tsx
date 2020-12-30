import React, { useEffect, useState } from "react";
import { NextProposalMessage, GameMessage, GameMessageType, ProposalUpdatedMessage, Vote, GameActionType, InteractionProps } from "../constants";
import { ProgressBar } from "react-bootstrap";
import { GameSocket, InboundMessage, InboundMessageType } from "../../../utils/GameSocket";
import { sendGameAction } from "../gameUtils";
import { PlayerCard } from "../playerCard";
import { createSelectedPlayerTypesList } from "../gameUtils";

/**
 * Props object for the ProposalManager
 */
interface ProposalManagerProps extends InteractionProps {
    message: NextProposalMessage,
    me: string,
    setPrimarySelectedPlayers: React.Dispatch<React.SetStateAction<Set<string>>>,
    setSecondarySelectedPlayers: React.Dispatch<React.SetStateAction<Set<string>>>,
    votes: Map<string, Vote>
}

/**
 * Handles all submissions and selections related to the proposal phase, including
 * the awful phase known as mission 1 second proposal.
 * @param props Required properties for the ProposalManager
 */
export function ProposalManager(props: ProposalManagerProps): JSX.Element {
    // State tracking if we're on the second proposal or not.
    const [onSecondM1Proposal, setOnSecondM1Proposal] = useState(false);

    // Refresh on props change to deal with stale closures...I guess.
    useEffect(() => {
        const connection = GameSocket.getInstance();
        connection.onGameEvent.subscribe(handleMessage);
        return () => connection.onGameEvent.unsubscribe(handleMessage);
    }, [props]);

    /**
     * Handles any incoming server messages.
     * @param message An inbound message from the server.
     */
    function handleMessage(message: InboundMessage): void {
        // Due to some issues with closures, derived variables like this must be
        // defined in the local function, not used from external scope.
        // Otherwise, React/JS fail to handle these correctly.
        const isProposing = props.message.proposer === props.me;
        if (message.messageType !== InboundMessageType.GameMessage) {
            return;
        }
        const gameMessage = message.data as GameMessage;
        switch (gameMessage.messageType) {
            case (GameMessageType.ProposalUpdated):
                // If we're proposing, the state is already kept locally, so don't
                // repeat computations.
                if (isProposing) {
                    return;
                }
                const proposalUpdated = gameMessage.data as ProposalUpdatedMessage;
                updateProposalFromServer(proposalUpdated.players);
                break;
            // If we're on mission 1, a proposal made indicates that we're on the second proposal
            case (GameMessageType.ProposalMade):
                if (props.message.mission === 1) {
                    setOnSecondM1Proposal(true);
                    break;
                }
        }
    }

    /**
     * Updates the selected players locally with the players received from the server,
     * taking care to handle mission 1.
     * @param incomingPlayers The selected players from the server
     */
    function updateProposalFromServer(incomingPlayers: string[]): void {
        const newSelectedPlayers = new Set(incomingPlayers);
        let setUpdater = onSecondM1Proposal ? props.setSecondarySelectedPlayers : props.setPrimarySelectedPlayers;
        setUpdater(newSelectedPlayers);
    }

    /**
     * Updates the set of selected players locally and sends these to the server.
     * @param name The player name to add/remove from the set.
     */
    function updateSelectedPlayers(name: string): void {
        // Use tempSet since react stateful variables must never be modified directly
        const setToUpdate = onSecondM1Proposal ? props.secondarySelectedPlayers : props.primarySelectedPlayers;
        const setUpdater = onSecondM1Proposal ? props.setSecondarySelectedPlayers : props.setPrimarySelectedPlayers;
        const tempSet = new Set(setToUpdate);
        if (!tempSet.delete(name)) {
            tempSet.add(name);
            sendGameAction(GameActionType.SelectPlayer, { player: name });
        } else {
            sendGameAction(GameActionType.UnselectPlayer, { player: name });
        }
        setUpdater(tempSet);
    }

    /**
     * Submits a proposal to the server using the correct proposal list.
     */
    function submitProposal(): void {
        const setToSubmit = onSecondM1Proposal ? props.secondarySelectedPlayers : props.primarySelectedPlayers;
        sendGameAction(GameActionType.Propose, { players: Array.from(setToSubmit) });
    }

    // Convienence variables unapcked from the props object.
    const { primarySelectedPlayers, secondarySelectedPlayers } = props;
    const { proposer, mission_size } = props.message;
    const isProposing = props.me === proposer;
    const selectedPlayers = onSecondM1Proposal ? secondarySelectedPlayers : primarySelectedPlayers;

    // Create the player cards
    const playerCards = props.playerList.map((playerName) => {
        const selectedTypes = createSelectedPlayerTypesList(playerName, primarySelectedPlayers, secondarySelectedPlayers);
        return <PlayerCard
            key={playerName}
            name={playerName}
            isProposing={playerName === proposer}
            tabbedOut={props.tabbedOutPlayers.has(playerName)}
            selectedTypes={selectedTypes}
            toggleSelected={updateSelectedPlayers}
            vote={props.votes.get(playerName)}
            enabled={isProposing && (selectedPlayers.size < mission_size || selectedPlayers.has(playerName))} />
    });


    return (
        <>
            {playerCards}
            <div className="proposal-manager">
                {!isProposing &&
                    <ProgressBar
                        now={selectedPlayers.size * 100 / mission_size}
                        label={`Selected: ${ selectedPlayers.size } / ${ mission_size }`} />}
                {isProposing &&
                    <button onClick={() => submitProposal()}>
                        Submit Proposal ({`${ selectedPlayers.size } / ${ mission_size }`})
                    </button>
                }
            </div>
        </>
    );
}