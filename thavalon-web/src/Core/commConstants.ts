import {MissionResult, Card, Vote, Team} from "./gameConstants.js";

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

export interface VoteStillInProgressMessage {
    submittedVote: Vote
}

export interface IConnectionManager {
    IsOpen: any
}


// Everything below here is legit.
export enum IncomingMessageTypes {
    RoleInformation,
    MissionResult,
    AllMissionInfo,
    PlayerOrder,
    VoteResult,
    NewProposal,
    ProposalReceived,
    MoveToVote
}

export enum OutgoingMessageTypes {
    RoleInformation,
    SubmitVote,
    AllMissionInfoRequest,

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