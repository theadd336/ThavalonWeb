import { GamePhase, MissionResult, Card } from "./gameConstants";

export enum IncomingMessageTypes {
    RoleInformation,
    MissionResult,
    AllMissionInfo,
    PlayerOrder,
    VoteResult,
    NewProposal,
    ProposalReceived,
    MoveToVote,
    AssassinationResponse,
    MissionInformation,
    GamePhaseChange,
    AbilityInformationResponse
}

export enum OutgoingMessageTypes {
    RoleInformation,
    SubmitVote,
    AllMissionInfoRequest,
    SubmitProposal,
    MoveToVote,
    SubmitAssassination,
    PlayerOrder,
    ProposalVoteInformationRequest,
    PlayCard,
    AbilityInformationRequest,
    UseAbility
}

export interface IncomingMessage {
    success: boolean;
    errorMessage: string;
    type: IncomingMessageTypes;
    data: object;
}

export interface OutgoingMessage {
    type: OutgoingMessageTypes;
    data?: object;
}

export interface GamePhaseChangeMessage {
    gamePhase: GamePhase;
}

export interface MissionResultsMessage {
    priorMissionNum: number;
    missionResult: MissionResult;
    playersOnMission: string[];
    playedCards: Card[];
}
