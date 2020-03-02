import {MissionResult, Card, Vote, Team} from "./gameConstants.js";

export interface WebSocketMessage {
    success: boolean;
    errorMessage: string;
    type: string;
    data: object;
}

export interface ProposalReceivedMessage {
    proposerName: string,
    proposedPlayerList: string[],
    isProposing: boolean
}

export interface MoveToVoteMessage {
    proposedPlayerList: string[]
}

export interface NewProposalMessage {
    isProposing: boolean,
    proposerIndex: number[],
    proposalNumber: number,
    maxNumProposals: number,
    proposalSize: number,
    currentProposal: string[],
    priorVoteInformation?: string[]
}

export interface MissionStartMessage {
    isOnMission: boolean,
    playersOnMissionList: string[],
    priorVoteInformation: string[]
}

export interface MissionResultsMessage {
    priorMissionNum: number,
    missionResult: MissionResult,
    playersOnMission: string[],
    playedCards: Card[]
}

export interface VoteStillInProgressMessage {
    submittedVote: Vote
}

export interface IConnectionManager {
    IsOpen: any
}


// Everything below here is legit.
export enum IncomingMessageTypes {
    RoleInformation = "role_information",
    MissionResult = "mission_result",
    AllMissionInfo = "all_mission_info",
}

export interface RoleInformationMessage {
    role: string,
    team: Team
}

export enum OutgoingMessageTypes {
    RoleInformation = "role_information",
}