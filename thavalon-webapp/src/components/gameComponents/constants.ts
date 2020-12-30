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
    Success = "Success",
    Fail = "Fail",
    Reverse = "Reverse",
    QuestingBeast = "Questing Beast <3"
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
    ProposalUpdated = "proposalUpdated",
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
    Play = "Play",
    Obscure = "Obscure",
    QuestingBeast = "QuestingBeast",
    Declare = "Declare",
    Assassinate = "Assassinate",
    MoveToAssassination = "MoveToAssassination"
}

/**
 * An interface for the game message from the server
 */
export interface GameMessage {
    messageType: GameMessageType,
    data?: object | string
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
    mission_size: number
}

export interface ProposalUpdatedMessage {
    players: string[],
}

export enum SelectedPlayerType {
    Primary,
    Secondary
}

export interface InteractionProps {
    primarySelectedPlayers: Set<string>,
    secondarySelectedPlayers: Set<string>,
    playerList: string[],
    tabbedOutPlayers: Set<string>,
}

export interface MissionGoingMessage {
    mission: number,
    players: string[],
}

export interface VotingResultsMessage {
    sent: boolean
    counts: VoteCounts
}

export interface VoteCounts {
    voteType: "Public" | "Private",
    upvotes: number | string[],
    downvotes: number | string[],
}

export interface MissionResultsMessage {
    mission: number,
    successes: number,
    fails: number,
    reverses: number,
    questing_beasts: number,
    passed: boolean
}