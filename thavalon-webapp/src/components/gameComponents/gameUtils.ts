import { GameMessageType, GameActionType } from "./constants";
import { GameSocket, OutboundMessage, OutboundMessageType } from "../../utils/GameSocket";

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
        case GameMessageType.NextProposal:
            gamePhase = GamePhase.Proposal;
            break;
        case GameMessageType.MissionGoing:
            gamePhase = GamePhase.Mission;
            break;
    }
    return gamePhase;
}

export function sendGameAction(actionType: GameActionType, data?: object | string | boolean | number): void {
    const connection = GameSocket.getInstance();
    const message: OutboundMessage = {
        messageType: OutboundMessageType.GameCommand,
        data: {
            messageType: actionType,
            data: data
        }
    }
    connection.sendMessage(message);
}