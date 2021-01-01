import React, { useEffect, useState } from "react";
import { GameSocket, InboundMessage, InboundMessageType } from "../../utils/GameSocket";
import { GameMessage, GameMessageType, MissionGoingMessage, MissionResultsMessage } from "./constants";
import "../../styles/gameStyles/missionResults.scss";
import { OverlayTrigger, Tooltip } from "react-bootstrap";

enum MissionStatus {
    Pending = "pending",
    Going = "going",
    Passed = "passed",
    Failed = "failed"
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

export function MissionResults(): JSX.Element {


    const [missionProps, setMissionProps] = useState<MissionCardProps[]>([
        {
            missionPlayers: ["Mission 1"],
            missionStatus: MissionStatus.Pending,
            missionNumber: 1,
        },
        {
            missionPlayers: ["Mission 2"],
            missionStatus: MissionStatus.Pending,
            missionNumber: 2,
        },
        {
            missionPlayers: ["Mission 3"],
            missionStatus: MissionStatus.Pending,
            missionNumber: 3,
        },
        {
            missionPlayers: ["Mission 4"],
            missionStatus: MissionStatus.Pending,
            missionNumber: 4,
        },
        {
            missionPlayers: ["Mission 5"],
            missionStatus: MissionStatus.Pending,
            missionNumber: 5,
        }
    ])

    // useEffect handles componentDidMount and componentWillUnmount steps.
    useEffect(() => {
        // On mount, get the connection instance and set up event handlers.
        // Then, get the player list.
        const connection = GameSocket.getInstance();
        connection.onGameEvent.subscribe(handleMessage);

        // create test missionGoing message
        const inboundMissionGoingMessage: InboundMessage = {
            messageType: InboundMessageType.GameMessage,
            data: {
                messageType: GameMessageType.MissionGoing,
                data: {
                    mission: 1,
                    players: new Set<String>(["Jared", "Paul", "Ben"]),                
                }
            }
        }

        connection.sendTestGameMessage(inboundMissionGoingMessage);

        const inboundMissionResultsMessage: InboundMessage = {
            messageType: InboundMessageType.GameMessage,
            data: {
                messageType: GameMessageType.MissionResults,
                data: {
                    mission: 1,
                    successes: 2,
                    fails: 3,
                    reverses: 2,
                    questing_beasts: 69,
                    passed: false,
                }
            }
        }

        setTimeout(() => connection.sendTestGameMessage(inboundMissionResultsMessage), 2000);

        // On unmount, unsubscribe our event handlers.
        return () => {
            const connection = GameSocket.getInstance();
            connection.onGameEvent.unsubscribe(handleMessage);
        }
    }, []);

    /**
     * Handles any game messages that come from the server. Currently supports
     * MissionGoing and MissionResults game messages.
     * @param message An incoming GameMessage from the server.
     */
    const handleGameMessage = (message: GameMessage) => {
        console.log(message);
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
                let missionResultsData = (message.data as MissionResultsMessage);
                let newArr = new Array(...missionProps);
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
        }
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
        }
    }

    console.log("RENDERING! Missiong props 0 is: ");
    console.log(missionProps[0]);

    return <div id="missionContainer">
        <h1 className="game-section-header">Mission Results</h1>
        <MissionCard {...missionProps[0]} />
        <MissionCard {...missionProps[1]} />
        <MissionCard {...missionProps[2]} />
        <MissionCard {...missionProps[3]} />
        <MissionCard {...missionProps[4]} />
    </div>;
}

/**
 * React component representing an interactive mission info. This button doesn't
 * directly communicate with the server but handles displaying of all mission info.
 * @param props The props for the mission card.
 */
function MissionCard(props: MissionCardProps): JSX.Element {
    function MissionTooltip(_: any): JSX.Element {
        return (
            <Tooltip id={`missionTooltip${props.missionNumber}`}>
                {props.missionStatus === MissionStatus.Pending && <div className="mission-tooltip">Mission {props.missionNumber} Pending</div>}
                {props.missionStatus === MissionStatus.Going && <div className="mission-tooltip">Mission {props.missionNumber} is going now</div>}
                {(props.missionStatus === MissionStatus.Passed || props.missionStatus === MissionStatus.Failed) && <div className="mission-tooltip">
                    Passes: {props.passes}<br />
                    Fails: {props.fails}<br />
                    Reverses: {props.reverses}<br />
                    Questing Beasts &lt;3: {props.questing_beasts}
                </div>}
            </Tooltip>
        );
    }    
    return (
        <OverlayTrigger
            placement="top"
            delay={{show: 250, hide: 400}}
            overlay={MissionTooltip}
        >
            <div className={`mission-card mission-${props.missionStatus}`} >
                {props.missionPlayers.join(" ")}
            </div>
        </OverlayTrigger>
    );
}