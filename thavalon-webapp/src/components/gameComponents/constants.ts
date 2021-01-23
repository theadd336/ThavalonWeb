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
    QuestingBeast = "Questing Beast was here <3"
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
    GameOver = "gameOver",
    Toast = "toast",
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
    MoveToAssassination = "MoveToAssassination",
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

/**
 * The next proposal message sent from the server.
 */
export interface NextProposalMessage {
    proposer: string,
    mission: number,
    proposals_made: number,
    max_proposals: number,
    mission_size: number
}

/**
 * The proposal updated message from the server
 */
export interface ProposalUpdatedMessage {
    players: string[],
}

/**
 * The type of selected player
 */
export enum SelectedPlayerType {
    Primary,
    Secondary
}

/**
 * The base props for all the interaction components
 */
export interface InteractionProps {
    primarySelectedPlayers: Set<string>,
    secondarySelectedPlayers: Set<string>,
    playerList: string[],
    tabbedOutPlayers: Set<string>,
    role: Role,
}

/**
 * The message of a mission going
 */
export interface MissionGoingMessage {
    mission: number,
    players: string[],
}

/**
 * The message representing the vote results
 */
export interface VotingResultsMessage {
    sent: boolean
    counts: VoteCounts
}

/**
 * The VoteCounts object from the VotingResultsMessage
 */
export interface VoteCounts {
    voteType: "Public" | "Private",
    upvotes: number | string[],
    downvotes: number | string[],
}

/**
 * Message representing mission results from the server.
 */
export interface MissionResultsMessage {
    mission: number,
    successes: number,
    fails: number,
    reverses: number,
    questing_beasts: number,
    passed: boolean
}

/**
 * Enum of roles in the game.
 */
export enum Role {
    Merlin = "Merlin",
    Lancelot = "Lancelot",
    Percival = "Percival",
    Tristan = "Tristan",
    Iseult = "Iseult",
    Mordred = "Mordred",
    Morgana = "Morgana",
    Maelegant = "Maelegant",
    Maeve = "Maeve",
    Agravaine = "Agravaine"
}

/** Length of time Agravaine has to declare in seconds */
export const AGRAVAINE_DECLARATION_TIME = 30;

/**
 * Interface for an Agravaine declaration message.
 */
export interface AgravaineDeclarationMessage {
    mission: number,
    player: string,
}
