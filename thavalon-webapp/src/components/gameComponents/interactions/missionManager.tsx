import React, { useEffect, useState } from "react";
import ReactModal from "react-modal";
import { GameSocket, InboundMessage, InboundMessageType } from "../../../utils/GameSocket";
import { GameActionType, InteractionProps, MissionCard, Vote, MissionGoingMessage, GameMessage, GameMessageType, MissionResultsMessage } from "../constants";
import { createSelectedPlayerTypesList, sendGameAction } from "../gameUtils";
import { PlayerCard } from "../playerCard";

import "../../../styles/gameStyles/interactionStyles/missionManager.scss";
import { ListGroup } from "react-bootstrap";
/**
 * The required properties for the MissionManager
 */
interface MissionManagerProps extends InteractionProps {
    me: string,
    message: MissionGoingMessage,
    votes: Map<string, Vote>,
}

/**
 * The required properties for the MissionCard
 */
interface MissionCardProps {
    submitMissionCard(card: MissionCard): void
}

/**
 * Required properties for the MissionResult modal.
 */
interface MissionResultModalProps {
    message: MissionResultsMessage | undefined,
    setOpen: React.Dispatch<React.SetStateAction<boolean>>
}

/**
 * Component managing all of the mission related interactions.
 * @param props Required properties for the MissionManager
 */
export function MissionManager(props: MissionManagerProps): JSX.Element {
    useEffect(() => {
        const connection = GameSocket.getInstance();
        connection.onGameEvent.subscribe(handleMessage);
        return () => connection.onGameEvent.unsubscribe(handleMessage);
    });

    /**
     * Handles any incoming server message
     * @param message A message from the server
     */
    function handleMessage(message: InboundMessage): void {
        if (message.messageType !== InboundMessageType.GameMessage) {
            return;
        }
    }

    // State maintaining if the player has played a card or not
    const [hasPlayedCard, setHasPlayedCard] = useState(false);

    /**
     * Submits a mission card or QB to the server and updates if the player
     * has played a card accordingly.
     * @param card A mission card to play
     */
    function submitMissionCard(card: MissionCard): void {
        if (card === MissionCard.QuestingBeast) {
            sendGameAction(GameActionType.QuestingBeast);
        } else {
            sendGameAction(GameActionType.Play, { card: card });
            setHasPlayedCard(true);
        }
    }

    // Create player cards here
    const playerCards = props.playerList.map((playerName) => {
        const selectedTypes = createSelectedPlayerTypesList(playerName, props.primarySelectedPlayers, props.secondarySelectedPlayers);
        return <PlayerCard
            key={playerName}
            name={playerName}
            vote={props.votes.get(playerName)}
            tabbedOut={props.tabbedOutPlayers.has(playerName)}
            selectedTypes={selectedTypes}
            enabled={false} />
    });

    const playersOnMission = new Set(props.message.players);
    return (
        <>
            {playerCards}
            <div className="interaction-manager">
                {!hasPlayedCard && playersOnMission.has(props.me) &&
                    <MissionCardButtons submitMissionCard={submitMissionCard} />}
                {(hasPlayedCard || !playersOnMission.has(props.me)) &&
                    <>Please wait for the mission cards to be played </>
                }
            </div>
        </>
    )

}

/**
 * Component managing the card buttons a player interacts with
 * @param props Required properties for the mission cards
 */
function MissionCardButtons(props: MissionCardProps): JSX.Element {
    const buttons = new Array<JSX.Element>();
    for (const card of Object.values(MissionCard)) {
        let className = "mission-button-success";
        switch (card) {
            case MissionCard.Fail:
                className = "mission-button-fail";
                break;
            case MissionCard.Reverse:
                className = "mission-button-reverse";
                break;
            case MissionCard.QuestingBeast:
                className = "mission-button-qb";
                break;
        }
        buttons.push(
            <button
                className={className}
                key={card}
                onClick={() => props.submitMissionCard(card)}>
                {card}
            </button>
        );
    }
    return (
        <div className="mission-card-button-row">
            {buttons}
        </div>
    )
}

/**
 * Component showing the results of a mission with Agravaine support
 * @param props Required properties for the mission result modal
 */
export function MissionResultModal(props: MissionResultModalProps): JSX.Element {
    if (props.message === undefined) {
        return <></>;
    }
    const { successes, fails, reverses, questing_beasts, mission } = props.message;
    return (
        <ReactModal
            isOpen={true}
            onRequestClose={() => props.setOpen(false)}
            contentLabel="Mission Result Modal"
            className="Modal"
            overlayClassName="Overlay"
        >
            <div className="modalContainer">
                <h2 className="modalHeader">
                    Mission {mission} <span className={props.message.passed ? "mission-header-passed" : "mission-header-failed"}>
                        {props.message.passed ? "Passed!" : "Failed!"}
                    </span>
                </h2>
                <hr />
                <ListGroup variant="flush">
                    <ListGroup.Item key={"successes"}>Successes: {successes}</ListGroup.Item>
                    <ListGroup.Item key={"fails"}>Fails: {fails}</ListGroup.Item>
                    <ListGroup.Item key={"reverses"}>Reverses: {reverses}</ListGroup.Item>
                    <ListGroup.Item key={"questing_beasts"}>Questing Beasts were here &lt;3: {questing_beasts}</ListGroup.Item>
                </ListGroup>
                <div>
                    <span className="waiting-for-agravaine">Agravaine has 30 seconds to declare.</span>
                    <button className="mission-modal-close-button" onClick={() => props.setOpen(false)}>Close</button>
                </div>
            </div>
        </ReactModal>
    );
}