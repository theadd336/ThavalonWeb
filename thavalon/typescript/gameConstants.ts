export enum Card {
    Success,
    Fail,
    Reverse
}

export const enum GamePhase {
    Proposal,
    Voting,
    Mission,
    Assassination
}

export const enum Vote {
    Downvote,
    Upvote
}

export const enum Team {
    Good,
    Evil
}

export const enum MissionResult {
    Pass,
    Fail
}

export interface RoleInformation {
    role: string;
    roleSummary: string;
    team: Team;
}

export interface AllMissionInfo {
    missionNumber: number;
    singleMissionInfo: missionInfo;
}

export interface missionInfo {
    missionResult: MissionResult;
    playersOnMission: string[];
    playedCards: Card[];
}

