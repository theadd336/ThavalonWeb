import React, { useEffect, useState } from "react";
import { GameSocket, InboundMessage, InboundMessageType, OutboundMessageType } from "../../utils/GameSocket";
import { Vote, GameMessageType, GameMessage, Snapshot, GameActionType, NextProposalMessage } from "./constants";
import { ProgressBar, Spinner } from "react-bootstrap";

import "../../styles/gameStyles/playerBoard.scss";
import { GamePhase, mapMessageToGamePhase, sendGameAction } from "./gameUtils";


/**
 * Props object for the PlayerCard component. Some of these aren't used yet,
 * pending future development.
 */
interface PlayerCardProps {
    name: string
    toggleSelected: (name: string) => void,
    me?: boolean,
    tabbedOut?: boolean,
    isProposing?: boolean,
    isSelected?: boolean,
    vote?: Vote,
    declaredAs?: string,
    enabled?: boolean
}

/**
 * Message for the tabbed out indicator message.
 */
interface PlayerFocusChangeMessage {
    displayName: string,
    isTabbedOut: boolean
}

interface ProposalUpdatedMessage {
    players: string[],
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
    const [selectedPlayers, setSelectedPlayers] = useState(new Set<string>());
    // State for maintaining players who are tabbed out. These players have a tab indicator.
    const [tabbedOutPlayers, setTabbedOutPlayers] = useState(new Set<string>());
    // State for maintaining the current game phase
    const [gamePhase, setGamePhase] = useState(GamePhase.Proposal);
    // State for maintaining the current mission number
    const [missionNumber, setMissionNumber] = useState(1);
    // State for tracking who this player is
    const [me, setMe] = useState("");
    // State for tracking the current proposer
    const [proposer, setProposer] = useState("");
    // State for maintaining the number of players on a proposal
    const [numPlayersOnProposal, setNumPlayersOnProposal] = useState(2);
    // State for maintaining the number of received votes
    const [numVotesReceived, setNumVotesReceived] = useState(0);

    /**
     * Generic message handler for all messages from the server
     * @param message The InboundMessage from the server.
     */
    function handleMessage(message: InboundMessage): void {
        switch (message.messageType) {
            case InboundMessageType.Snapshot:
                const snapshot = message.data as Snapshot
                setMe(snapshot.me);
                handleGameMessage(snapshot.log[0]);
                for (let i = snapshot.log.length - 1; i >= 0; i--) {
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
                setProposer(data.proposer);
                setMissionNumber(data.mission);
                setNumPlayersOnProposal(data.mission_size);
                break;
            case GameMessageType.ProposalUpdated:
                const proposalUpdatedMessage = message.data as ProposalUpdatedMessage;
                const newPlayers = new Set(proposalUpdatedMessage.players);
                setSelectedPlayers(newPlayers);
                break;
            case GameMessageType.VoteRecieved:
                setNumVotesReceived(numVotesReceived + 1);
                break;
        }
    }

    /**
     * Updates the selected player list with a new name.
     * @param name The player name to select.
     */
    function updateSelectedPlayers(name: string): void {
        // Use tempSet since react stateful variables must never be modified directly
        const tempSet = new Set(selectedPlayers.values());
        if (!tempSet.delete(name)) {
            tempSet.add(name);
            sendGameAction(GameActionType.SelectPlayer, { player: name });
        } else {
            sendGameAction(GameActionType.UnselectPlayer, { player: name });
        }
        setSelectedPlayers(tempSet);
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

    // Create the player cards with the state we have.
    const playerCards = playerList.map((playerName) => {
        return (
            <PlayerCard
                key={playerName}
                name={playerName}
                tabbedOut={tabbedOutPlayers.has(playerName)}
                isSelected={selectedPlayers.has(playerName)}
                toggleSelected={updateSelectedPlayers}
                // TODO: Support roles like guin and oberon here as well as assassination.
                enabled={gamePhase === GamePhase.Proposal && (proposer === me)} />
        );
    });

    return (
        <div className="player-board">
            {playerCards}
            {
                gamePhase === GamePhase.Proposal &&
                <ProposalInteractions
                    isProposing={proposer === me}
                    numSelectedPlayers={selectedPlayers.size}
                    numPlayersOnProposal={numPlayersOnProposal}
                    submitProposal={() => sendGameAction(GameActionType.Propose, { players: Array.from(selectedPlayers) })}
                />
            }
            {
                gamePhase === GamePhase.Vote &&
                <VoteButtonGroup
                    submitVote={(vote: Vote) => {
                        sendGameAction(GameActionType.Vote, { upvote: Boolean(vote) });
                    }}
                    isFirstMission={missionNumber === 1}
                    playerCount={playerList.length}
                    votesReceived={numVotesReceived} />
            }
        </div>
    );
}

/**
 * React component representing an interactive player button. This button doesn't
 * directly communicate with the server but handles all styling of relevant icons.
 * @param props The props for the player card
 */
function PlayerCard(props: PlayerCardProps): JSX.Element {
    let disabled = false;
    if (props.enabled === false) {
        disabled = true;
    }
    return (
        <button
            disabled={disabled}
            className={`player-card ${ props.isSelected ? "player-selected" : "" }`}
            onClick={() => props.toggleSelected(props.name)}>
            <div>
                {props.tabbedOut &&
                    <Spinner
                        className="tabbed-out-indicator"
                        size="sm"
                        variant="dark"
                        animation="border" />}
                {props.name}
            </div>
        </button>
    );
}

//#region Voting

interface VoteButtonGroupProps {
    submitVote: (vote: Vote) => void,
    isFirstMission: boolean,
    playerCount: number,
    votesReceived: number,
}

function VoteButtonGroup(props: VoteButtonGroupProps): JSX.Element {
    const [votesReceived, setVotesReceived] = useState(0);
    const [hasVoted, setHasVoted] = useState(false);
    useEffect(() => {
        const connection = GameSocket.getInstance();
        connection.onGameEvent.subscribe(handleMessage);
        return () => connection.onGameEvent.unsubscribe(handleMessage);
    }, []);

    function handleMessage(message: InboundMessage): void {
        if (message.messageType !== InboundMessageType.GameMessage) {
            return;
        }
        const gameMessage = message.data as GameMessage;
        if (gameMessage.messageType === GameMessageType.VoteRecieved) {
            setVotesReceived(votesReceived + 1);
        }
    }

    return (
        <div className="vote-button-group">
            {hasVoted ?
                <VoteCount
                    votesReceived={votesReceived}
                    playerCount={props.playerCount} />
                :
                <VoteButtons
                    isFirstMission={props.isFirstMission}
                    submitVote={props.submitVote}
                    setHasVoted={setHasVoted} />}
        </div>
    );
}

interface VoteCountProps {
    votesReceived: number,
    playerCount: number
}

function VoteCount(props: VoteCountProps): JSX.Element {
    return (
        <ProgressBar now={props.votesReceived * 100 / props.playerCount} label={`${ props.votesReceived } / ${ props.playerCount }`} />
    )
}

interface VoteButtonProps {
    isFirstMission: boolean,
    submitVote: (vote: Vote) => void,
    setHasVoted: React.Dispatch<React.SetStateAction<boolean>>
}

function VoteButtons(props: VoteButtonProps): JSX.Element {
    function handleVoteSubmit(vote: Vote): void {
        props.submitVote(vote);
        props.setHasVoted(true);
    }
    return (
        <>
            <button
                className="vote-button green"
                onClick={() => handleVoteSubmit(Vote.Upvote)}>
                {props.isFirstMission ? "Send Green" : "Accept"}
            </button>
            <button
                className="vote-button red"
                onClick={() => handleVoteSubmit(Vote.Downvote)}>
                {props.isFirstMission ? "Send Red" : "Accept"}
            </button>
        </>
    );
}

//#endregion

//#region Proposals
interface ProposalInteractionsProps {
    isProposing: boolean,
    numPlayersOnProposal: number,
    numSelectedPlayers: number,
    submitProposal: () => void,
}


export function ProposalInteractions(props: ProposalInteractionsProps): JSX.Element {
    return (
        <div className="proposal-interactions">
            {!props.isProposing &&
                <ProgressBar
                    now={props.numSelectedPlayers * 100 / props.numPlayersOnProposal}
                    label={`Selected: ${ props.numSelectedPlayers } / ${ props.numPlayersOnProposal }`} />}
            {props.isProposing &&
                <button onClick={() => props.submitProposal()}>
                    Submit Proposal ({`${ props.numSelectedPlayers } / ${ props.numPlayersOnProposal }`})
                </button>
            }
        </div>
    );
}

//#endregion
