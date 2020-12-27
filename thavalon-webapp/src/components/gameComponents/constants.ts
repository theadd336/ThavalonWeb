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


export enum GameMessageType {
    Error = "Error",
    ProposalOrder = "proposalOrder",
    RoleInformation = "roleInformation",
    NextProposal = "nextProposal",
    ProposalMade = "proposalMade",
    CommenceVoting = "commenceVoting",
    MissionGoing = "missionGoing",
    MissionResults = "missionResults",
    AgravaineDeclaration = "agravaineDeclaration",
    BeginAssassination = "beginAssassination",
    AssassinationResult = "assassinationResult",
    GameOver = "gameOver"
}

export interface GameMessage {
    messageType: GameMessageType,
    data?: object | string
}
