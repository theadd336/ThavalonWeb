import React, { useEffect, useState } from "react";
import ReactModal from "react-modal";
import { GameSocket, InboundMessage, InboundMessageType } from "../../../utils/GameSocket";
import { GameActionType, InteractionProps, MissionCard, Vote, MissionGoingMessage, GameMessage, GameMessageType, MissionResultsMessage } from "../constants";
import { createSelectedPlayerTypesList, sendGameAction } from "../gameUtils";
import { PlayerCard } from "../playerCard";

interface MissionManagerProps extends InteractionProps {
    me: string,
    message: MissionGoingMessage,
    votes: Map<string, Vote>,
}

interface MissionCardProps {
    submitMissionCard(card: MissionCard): void
}

export function MissionManager(props: MissionManagerProps): JSX.Element {
    useEffect(() => {
        const connection = GameSocket.getInstance();
        connection.onGameEvent.subscribe(handleMessage);
        return () => connection.onGameEvent.unsubscribe(handleMessage);
    })

    function handleMessage(message: InboundMessage): void {
        if (message.messageType !== InboundMessageType.GameMessage) {
            return;
        }
        const gameMessage = message.data as GameMessage;
        if (gameMessage.messageType === GameMessageType.MissionResults) {
            setMissionResults(gameMessage.data as MissionResultsMessage);
            setShowMissionResults(true);
        }
    }
    const [hasPlayedCard, setHasPlayedCard] = useState(false);
    const [showMissionResults, setShowMissionResults] = useState(false);
    const [missionResults, setMissionResults] = useState<MissionResultsMessage>();

    function submitMissionCard(card: MissionCard): void {
        if (card === MissionCard.QuestingBeast) {
            sendGameAction(GameActionType.QuestingBeast);
        } else {
            sendGameAction(GameActionType.Play, { card: card });
            setHasPlayedCard(true);
        }
    }

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
            <div className="mission-manager">
                {!hasPlayedCard && playersOnMission.has(props.me) &&
                    <MissionCardButtons submitMissionCard={submitMissionCard} />}
                {(hasPlayedCard || !playersOnMission.has(props.me)) &&
                    <></>
                }
                {showMissionResults &&
                    <MissionResultModal message={missionResults} setOpen={setShowMissionResults} />
                }
            </div>
        </>
    )

}

function MissionCardButtons(props: MissionCardProps): JSX.Element {
    const buttons = new Array<JSX.Element>();
    for (const card of Object.values(MissionCard)) {
        buttons.push(
            <button
                key={card}
                onClick={() => props.submitMissionCard(card)}>
                {card}
            </button>
        );
    }
    return (
        <div className="mission-card-buttons">
            {buttons}
        </div>
    )
}

interface MissionResultProps {
    message: MissionResultsMessage | undefined,
    setOpen: React.Dispatch<React.SetStateAction<boolean>>
}
function MissionResultModal(props: MissionResultProps): JSX.Element {
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
                <h2 className="modalHeader">Misison {mission} {props.message.passed ? "Passed!" : "Failed!"}</h2>
                <hr />
                <ul>
                    <li key={"successes"}>Successes: {successes}</li>
                    <li key={"fails"}>Fails: {fails}</li>
                    <li key={"reverses"}>Reverses: {reverses}</li>
                    <li key={"questing_beasts"}>Questing Beasts {"<"}3: {questing_beasts}</li>
                </ul>
                <span className="waiting-for-agravain">Waiting for Agravaine to declare...</span>
                <button onClick={() => props.setOpen(false)}>Close</button>
            </div>
        </ReactModal >
    );
}