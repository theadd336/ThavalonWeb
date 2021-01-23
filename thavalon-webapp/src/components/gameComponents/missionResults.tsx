import React, { useEffect, useState } from "react";
import { GameSocket, InboundMessage, InboundMessageType } from "../../utils/GameSocket";
import { GameMessage, GameMessageType, MissionGoingMessage, MissionResultsMessage, Snapshot } from "./constants";
import { OverlayTrigger, Tooltip } from "react-bootstrap";
import "../../styles/gameStyles/missionResults.scss";

enum MissionStatus {
    Pending = "pending",
    Going = "going",
    Passed = "passed",
    Failed = "failed",
}

interface MissionCardProps {
    missionPlayers: String[],
    missionStatus: MissionStatus,
    missionNumber: number,
    passes?: number,
    fails?: number,
    questing_beasts?: number,
    reverses?: number,
}

/**
 * MissionResults panel on game page, for showing mission results.
 */
export function MissionResults(): JSX.Element {
    const [missionProps, setMissionProps] = useState<MissionCardProps[]>([1, 2, 3, 4, 5].map((x) => (
        {
            missionPlayers: ["Mission " + x],
            missionStatus: MissionStatus.Pending,
            missionNumber: x,
        }
    )));

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
    }, [missionProps]);

    /**
     * Handles any game messages that come from the server. Currently supports
     * MissionGoing and MissionResults game messages.
     * @param message An incoming GameMessage from the server.
     */
    function handleGameMessage(message: GameMessage) {
        switch (message.messageType) {
            case GameMessageType.MissionGoing: {
                const missionGoingData = (message.data as MissionGoingMessage);
                const newArr = new Array(...missionProps);
                newArr[missionGoingData.mission - 1] = {
                    missionPlayers: Array.from(missionGoingData.players),
                    missionStatus: MissionStatus.Going,
                    missionNumber: missionGoingData.mission,
                }
                setMissionProps(newArr);
                break;
            }
            case GameMessageType.MissionResults: {
                const missionResultsData = message.data as MissionResultsMessage;
                const newArr = new Array(...missionProps);
                newArr[missionResultsData.mission - 1] = {
                    missionPlayers: missionProps[missionResultsData.mission - 1].missionPlayers,
                    missionStatus: missionResultsData.passed ? MissionStatus.Passed : MissionStatus.Failed,
                    missionNumber: missionResultsData.mission,
                    passes: missionResultsData.successes,
                    fails: missionResultsData.fails,
                    reverses: missionResultsData.reverses,
                    questing_beasts: missionResultsData.questing_beasts,
                }
                setMissionProps(newArr);
                break;
            }
            case GameMessageType.AgravaineDeclaration:
                console.log("In Agravaine declaration handling.");
                const newArr = new Array(...missionProps);
                for (let i = newArr.length - 1; i >= 0; i--) {
                    if (newArr[i].missionStatus !== MissionStatus.Pending) {
                        console.log("Setting mission " + i + " to failed.");
                        newArr[i].missionStatus = MissionStatus.Failed;
                        i = -1;
                    }
                }
                setMissionProps(newArr);
                break;
        }
    }

    function handleSnapshotMessage(snapshot: Snapshot) {
        const newArr = new Array(...missionProps);
        for (let i = 0; i < snapshot.missions.length; i++) {
            const mission = snapshot.missions[i];
            const sentProposal = mission.sentProposal;
            if (sentProposal === null) {
                continue;
            }
            newArr[i].missionPlayers = mission.proposals[sentProposal].players;
            const results = mission.results;
            if (results === null) {
                continue;
            }
            newArr[i].missionStatus = results.passed ? MissionStatus.Passed : MissionStatus.Failed;
            newArr[i].passes = results.successes;
            newArr[i].fails = results.fails;
            newArr[i].reverses = results.reverses;
            newArr[i].questing_beasts = results.questing_beasts;
        }
        setMissionProps(newArr);
    }

    /**
     * Handles any lobby messages that come from the server. If the message type
     * is a GameMessage, calls helper function handleGameMessage to handle it.
     * @param message An incoming message from the server.
     */
    function handleMessage(message: InboundMessage): void {
        switch (message.messageType) {
            case InboundMessageType.GameMessage: {
                handleGameMessage(message.data as GameMessage);
                break;
            }
            case InboundMessageType.Snapshot: {
                handleSnapshotMessage(message.data as Snapshot);
                break;
            }
        }
    }

    return <div id="missionContainer">
        <h1 className="game-section-header">Mission Results</h1>
        {missionProps.map(mission => <MissionCard {...mission} />)}
    </div>;
}

/**
 * React component representing an interactive mission info. This button doesn't
 * directly communicate with the server but handles displaying of all mission info.
 * @param props The props for the mission card.
 */
function MissionCard(props: MissionCardProps): JSX.Element {
    const missionTooltip = <Tooltip className="mission-tooltip" id={`missionTooltip${ props.missionNumber }`}>
        {props.missionStatus === MissionStatus.Pending && <div className="mission-tooltip">Mission {props.missionNumber} Pending</div>}
        {props.missionStatus === MissionStatus.Going && <div className="mission-tooltip">Mission {props.missionNumber} is going now</div>}
        {(props.missionStatus === MissionStatus.Passed || props.missionStatus === MissionStatus.Failed) && <div className="mission-tooltip">
            Passes: {props.passes}<br />
            Fails: {props.fails}<br />
            Reverses: {props.reverses}<br />
            Questing Beasts: {props.questing_beasts}
        </div>}
    </Tooltip>

    return (
        <OverlayTrigger
            placement="top"
            delay={{ show: 250, hide: 400 }}
            overlay={missionTooltip}
        >
            <div className={`mission-card mission-${ props.missionStatus }`} >
                {props.missionPlayers.join(" ")}
            </div>
        </OverlayTrigger>
    );
}