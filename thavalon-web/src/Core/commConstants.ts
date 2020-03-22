export enum IncomingMessageTypes {
    RoleInformation,
    MissionResult,
    AllMissionInfo,
    PlayerOrder,
    VoteResult,
    NewProposal,
    ProposalReceived,
    MoveToVote,
    AssassinationResponse
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
    PlayCard
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