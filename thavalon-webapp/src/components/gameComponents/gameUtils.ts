import { GameMessageType, GameActionType } from "./constants";
import { GameSocket, OutboundMessage, OutboundMessageType } from "../../utils/GameSocket";
import { SelectedPlayerType } from "./constants";

/**
 * Enum representing the four main phases of the game.
 */
export enum GamePhase {
    Proposal,
    Vote,
    Mission,
    Assassination
}

/**
 * Given a message, determines which phase of the game the player is currently in.
 * @param messageType A type of GameMessage from the server
 * @param priorGamePhase The previous game phase. 
 * Used as a default if none of the messages match, if provided
 */
export function mapMessageToGamePhase(messageType: GameMessageType): GamePhase {
    let gamePhase = GamePhase.Proposal;
    switch (messageType) {
        case GameMessageType.BeginAssassination:
            gamePhase = GamePhase.Assassination;
            break;
        case GameMessageType.VoteRecieved:
        case GameMessageType.CommenceVoting:
            gamePhase = GamePhase.Vote;
            break;
        case GameMessageType.ProposalMade:
        case GameMessageType.ProposalOrder:
        case GameMessageType.ProposalUpdated:
        case GameMessageType.NextProposal:
            gamePhase = GamePhase.Proposal;
            break;
        case GameMessageType.MissionGoing:
        case GameMessageType.MissionResults:
            gamePhase = GamePhase.Mission;
            break;
    }
    return gamePhase;
}

/**
 * Sends an Action to the server with any specified data
 * @param actionType The type of action to send to the server
 * @param data The data the action requires, if any.
 */
export function sendGameAction(actionType: GameActionType, data?: object | string | boolean | number): void {
    const connection = GameSocket.getInstance();
    const message: OutboundMessage = {
        messageType: OutboundMessageType.GameCommand,
        data: { [actionType]: data }
    };
    if (data === undefined) {
        message.data = actionType;
    }
    connection.sendMessage(message);
}

/**
 * Creates an array of selected player types given a player name and selected player sets.
 * @param name The player name to select
 * @param primarySelectedPlayers The primary selected players set
 * @param secondarySelectedPlayers The secondary selected players set
 */
export function createSelectedPlayerTypesList(
    name: string,
    primarySelectedPlayers: Set<string>,
    secondarySelectedPlayers: Set<string>):
    SelectedPlayerType[] {
    const selectedTypes = new Array<SelectedPlayerType>();
    if (primarySelectedPlayers.has(name)) {
        selectedTypes.push(SelectedPlayerType.Primary);
    }
    if (secondarySelectedPlayers.has(name)) {
        selectedTypes.push(SelectedPlayerType.Secondary);
    }
    return selectedTypes;
}