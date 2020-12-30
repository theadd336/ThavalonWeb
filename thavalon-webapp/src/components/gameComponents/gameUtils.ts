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
        // Technically not needed, but it's nice to explicitly map message types.
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

export function sendGameAction(actionType: GameActionType, data?: object | string | boolean | number): void {
    const connection = GameSocket.getInstance();
    let message: OutboundMessage;
    if (data === undefined) {
        message = {
            messageType: OutboundMessageType.GameCommand,
            data: actionType
        };
    } else {
        message = {
            messageType: OutboundMessageType.GameCommand,
            data: { [actionType]: data }
        }
    }
    connection.sendMessage(message);
}

export function createSelectedPlayerTypesList(
    name: string,
    primarySelectedPlayers: Set<string>,
    secondarySelectedPlayers: Set<string>): SelectedPlayerType[] {
    const selectedTypes = new Array<SelectedPlayerType>();
    if (primarySelectedPlayers.has(name)) {
        selectedTypes.push(SelectedPlayerType.Primary);
    }
    if (secondarySelectedPlayers.has(name)) {
        selectedTypes.push(SelectedPlayerType.Secondary);
    }
    return selectedTypes;
}