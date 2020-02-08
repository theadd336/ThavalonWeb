namespace ThavalonWeb.Game.Constants {
    export enum Card {
        Success,
        Fail,
        Reverse
    }

    export enum GamePhase {
        Proposal,
        Voting,
        Mission,
        Assassination
    }

    export enum Vote {
        Downvote,
        Upvote
    }

    export enum Team {
        Good,
        Evil
    }

    export enum MissionResult {
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
}

