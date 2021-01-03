import React, { useEffect, useState } from "react";
import { GameSocket, InboundMessage, InboundMessageType, OutboundMessageType } from "../../utils/GameSocket";
import { Vote, GameMessageType, GameMessage, Snapshot, NextProposalMessage, MissionGoingMessage, VotingResultsMessage, MissionResultsMessage } from "./constants";
import { ProposalManager } from "./interactions/proposalManager";
import { GamePhase, mapMessageToGamePhase } from "./gameUtils";
import { VoteManager } from "./interactions/voteManager";
import { MissionManager, MissionResultModal } from "./interactions/missionManager";
import "../../styles/gameStyles/playerBoard.scss";

/**
 * Message for the tabbed out indicator message.
 */
interface PlayerFocusChangeMessage {
    displayName: string,
    isTabbedOut: boolean
}

/**
 * Board containing the player list and all related functions. This is a container
 * for all PlayerCards and has handlers for game and lobby events.
 */
export function PlayerBoard(): JSX.Element {
    // On mount, set up event handlers for the game socket and the DOM.
    // On unmount, clean up event handlers.
    useEffect(() => {
        const connection = GameSocket.getInstance();
        connection.onGameEvent.subscribe(handleMessage);
        connection.onLobbyEvent.subscribe(handleMessage);
        document.onvisibilitychange = () => sendPlayerVisibilityChange();
        return () => {
            connection.onGameEvent.unsubscribe(handleMessage);
            document.onvisibilitychange = null;
        }
    }, [])

    // State for maintaining the player list.
    const [playerList, setPlayerList] = useState<string[]>([])
    // State maintaining selected players. These players are highlighted in green.
    const [primarySelectedPlayers, setPrimarySelectedPlayers] = useState(new Set<string>());
    // State maintaining the secondary selected players. These players are highlighted in red.
    const [secondarySelectedPlayers, setSecondarySelectedPlayers] = useState(new Set<string>());
    // State for maintaining players who are tabbed out. These players have a tab indicator.
    const [tabbedOutPlayers, setTabbedOutPlayers] = useState(new Set<string>());
    // State for maintaining the current game phase
    const [gamePhase, setGamePhase] = useState(GamePhase.Proposal);
    // State for maintaining the current mission number
    const [missionNumber, setMissionNumber] = useState(1);
    // State for tracking who this player is
    const [me, setMe] = useState("");
    // State for maintaining the last major message, initialized to a default NextProposalMessage
    const [majorMessage, setMajorMessage] = useState<NextProposalMessage | MissionGoingMessage>(
        {
            proposer: "",
            mission: 1,
            mission_size: 2,
            max_proposals: 1,
            proposals_made: 0
        }
    );
    // State for maintaining the map of players to votes
    const [votes, setVotes] = useState<Map<string, Vote>>(new Map<string, Vote>());
    // State to show the mission modal or not.
    const [showMissionResults, setShowMissionResults] = useState(false);
    // State to maintain the mission results.
    const [missionResults, setMissionResults] = useState<MissionResultsMessage>();

    /**
     * Generic message handler for all messages from the server
     * @param message The InboundMessage from the server.
     */
    function handleMessage(message: InboundMessage): void {
        switch (message.messageType) {
            case InboundMessageType.Snapshot:
                const snapshot = message.data as Snapshot
                setMe(snapshot.me);
                // Get proposal order, then get the most recent major message.
                // Finally, feed the last message in
                handleGameMessage(snapshot.log[0]);
                for (let i = snapshot.log.length - 2; i >= 1; i--) {
                    const logMessage = snapshot.log[i];
                    switch (logMessage.messageType) {
                        case GameMessageType.NextProposal:
                        case GameMessageType.CommenceVoting:
                        case GameMessageType.MissionGoing:
                        case GameMessageType.BeginAssassination:
                            handleGameMessage(logMessage);
                            i = -1;
                            break;
                    }
                }
                handleGameMessage(snapshot.log[snapshot.log.length - 1]);
                break;
            case InboundMessageType.PlayerFocusChange:
                const { displayName, isTabbedOut } = message.data as PlayerFocusChangeMessage;
                playerFocusChanged(displayName, isTabbedOut);
                break;
            case InboundMessageType.GameMessage:
                handleGameMessage(message.data as GameMessage);
                break;
        }
    }

    /**
     * GameMessage specific message handler. This is needed because the GameMessage
     * has internal types to parse.
     * @param message The GameMessage from the server
     */
    function handleGameMessage(message: GameMessage): void {
        setGamePhase(mapMessageToGamePhase(message.messageType));
        switch (message.messageType) {
            case GameMessageType.ProposalOrder:
                setPlayerList(message.data as string[]);
                break;
            case GameMessageType.NextProposal:
                const data = message.data as NextProposalMessage;
                if (data.mission > 1) {
                    setPrimarySelectedPlayers(new Set());
                    setSecondarySelectedPlayers(new Set());
                }
                setMissionNumber(data.mission);
                setMajorMessage(data);
                break;
            case GameMessageType.VotingResults:
                // If results are public, set them here. Otherwise, trigger toast for Maeve.
                const votingResults = message.data as VotingResultsMessage;
                if (votingResults.counts.voteType === "Public") {
                    const { upvotes, downvotes } = votingResults.counts;
                    if (typeof upvotes === "number" || typeof downvotes === "number") {
                        throw new TypeError("Votes for public votes must be string arrays, not numbers.");
                    }
                    const voteMap = new Map<string, Vote>();
                    for (const player of upvotes) {
                        voteMap.set(player, Vote.Upvote);
                    }
                    for (const player of downvotes) {
                        voteMap.set(player, Vote.Downvote);
                    }
                    console.log(voteMap);
                    setVotes(voteMap);
                } else {
                    // TODO: Trigger Maive Toast here.
                }
                break;
            case GameMessageType.MissionGoing:
                setMajorMessage(message.data as MissionGoingMessage);
                break;
            case GameMessageType.MissionResults:
                setShowMissionResults(true);
                setMissionResults(message.data as MissionResultsMessage);
        }
    }

    /**
     * Sends a message to the server that the player is tabbed in or out.
     */
    function sendPlayerVisibilityChange(): void {
        const connection = GameSocket.getInstance();
        const isTabbedOut = document.visibilityState === "hidden" ? true : false;
        const message = { messageType: OutboundMessageType.PlayerFocusChange, data: isTabbedOut };
        connection.sendMessage(message);
    }

    /**
     * Updates the tabbed out player list with a new player and visibility
     * @param player The player whose focus has changed
     * @param visibility The new visibility for that player
     */
    function playerFocusChanged(player: string, isTabbedOut: boolean): void {
        const tempSet = new Set(tabbedOutPlayers.values());
        if (isTabbedOut) {
            tempSet.add(player);
        } else {
            tempSet.delete(player);
        }
        setTabbedOutPlayers(tempSet);
    }

    return (
        <div className="player-board">
            {
                gamePhase === GamePhase.Proposal &&
                <ProposalManager
                    message={majorMessage as NextProposalMessage}
                    me={me}
                    playerList={playerList}
                    tabbedOutPlayers={tabbedOutPlayers}
                    primarySelectedPlayers={primarySelectedPlayers}
                    setPrimarySelectedPlayers={setPrimarySelectedPlayers}
                    secondarySelectedPlayers={secondarySelectedPlayers}
                    setSecondarySelectedPlayers={setSecondarySelectedPlayers}
                    votes={votes}
                />
            }
            {
                gamePhase === GamePhase.Vote &&
                <VoteManager
                    isMissionOne={missionNumber === 1}
                    playerList={playerList}
                    primarySelectedPlayers={primarySelectedPlayers}
                    secondarySelectedPlayers={secondarySelectedPlayers}
                    tabbedOutPlayers={tabbedOutPlayers} />
            }
            {
                gamePhase === GamePhase.Mission &&
                <MissionManager
                    me={me}
                    playerList={playerList}
                    message={majorMessage as MissionGoingMessage}
                    primarySelectedPlayers={primarySelectedPlayers}
                    secondarySelectedPlayers={secondarySelectedPlayers}
                    tabbedOutPlayers={tabbedOutPlayers}
                    votes={votes} />
            }
            {
                showMissionResults &&
                <MissionResultModal setOpen={setShowMissionResults} message={missionResults} />
            }
        </div>
    );
}