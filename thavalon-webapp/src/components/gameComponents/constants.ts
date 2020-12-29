/**
 * An enum of possible teams the player is on.
 */
export enum Team {
    Good = "Good",
    Evil = "Evil",
}

/**
 * An enum of cards to play
 */
export enum MissionCard {
    Pass = "Pass",
    Fail = "Fail",
    Reverse = "Reverse",
    QuestingBeast = "QuestingBeast",
}

/**
 * An enum of votes that a player can use. Note that this is technically an object
 * literal, since enums don't support booleans, but the server expects them.
 */
export enum Vote {
    Downvote,
    Upvote
}

/**
 * An enum of all GameMessage types
 */
export enum GameMessageType {
    Error = "error",
    ProposalOrder = "proposalOrder",
    RoleInformation = "roleInformation",
    NextProposal = "nextProposal",
    ProposalMade = "proposalMade",
    CommenceVoting = "commenceVoting",
    VoteRecieved = "voteReceived",
    VotingResults = "votingResults",
    MissionGoing = "missionGoing",
    MissionResults = "missionResults",
    AgravaineDeclaration = "agravaineDeclaration",
    BeginAssassination = "beginAssassination",
    AssassinationResult = "assassinationResult",
    GameOver = "gameOver"
}

/**
 * An interface for the game message from the server
 */
export interface GameMessage {
    messageType: GameMessageType,
    data?: object | string
}

export enum GameActionType {
    Propose = "Propose",
    SelectPlayer = "SelectPlayer",
    UnselectPlayer = "UnselectPlayer",
    Vote = "Vote",
    Obscure = "Obscure",
    QuestingBeast = "QuestingBeast",
    Declare = "Declare",
    Assassinate = "Assassinate",
    MoveToAssassination = "MoveToAssassination"
}

export interface GameAction {
    actionType: GameActionType,
    data?: any
}

/**
 * The role info provided by the server in the snapshot.
 */
export interface RoleInfo {
    abilities: string,
    assassinatable: boolean,
    description: string,
    isAssassin: boolean,
    otherInfo: string,
    priorityTarget: string | undefined,
    role: string,
    seenPlayers: string[],
    team: string,
    teamMembers: string[],
}

/**
 * The snapshot message sent by the server.
 */
export interface Snapshot {
    me: string,
    roleInfo: RoleInfo,
    missions: any[],
    log: GameMessage[],
}

export interface NextProposalMessage {
    proposer: string,
    mission: number,
    proposals_made: number,
    max_proposals: number,
}