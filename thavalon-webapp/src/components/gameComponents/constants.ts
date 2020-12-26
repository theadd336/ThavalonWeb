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
 * An enum of votes that a player can use
 */
export enum Vote {
    Upvote = "Upvote",
    Downvote = "Downvote",
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
    roleInfo: RoleInfo,
    missions: any[],
    log: any[],
}

