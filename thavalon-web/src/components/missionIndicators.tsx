import * as React from "react";
import { Popover, OverlayTrigger } from "react-bootstrap";
import { MissingPropertyError } from "../Core/errors";
import { MissionResult, Card } from "../Core/gameConstants";

/**
 * Handles mission indicator images and popovers.
 */
export class MissionIndicator extends React.Component<MissionIndicatorProps> {

    /**
     * Initializes the indicator and sets state if needed on reconnect
     * @param props Properties object for specific mission. 
     */
    constructor(props: MissionIndicatorProps) {
        super(props);
        const missionNum = this.props.missionNum;
        const playersOnMission = this.props.playersOnMission;
        const cardsPlayed = this.props.cardsPlayed;
        const result = this.props.result;

        if (missionNum === undefined
            || playersOnMission === undefined 
            || result === undefined
            || cardsPlayed === undefined) {
        
            throw new MissingPropertyError("A resulted mission must contain a result, players, and the cards played.");
        }
    }

    render(): React.ReactNode {
        return this.formatImageLink();
    }

    //#region private methods
    /**
     * Formats the correct mission indicator and popover for display
     */
    private formatImageLink(): React.ReactNode {
        // Extract the relavent props and state for easier access.
        const { missionNum, playersOnMission, cardsPlayed, result } = this.props;

        // Select the correct image source based on the mission result.
        const indicatorImageSource = this.selectIndicatorImage(result);

        // Initialize the popover and return that node.
        const popover = this.initializePopover(missionNum, playersOnMission, cardsPlayed, result);
        const indicatorNode = (
            <OverlayTrigger 
                trigger="click"
                placement="top"
                overlay={popover}>
                <img 
                    src={indicatorImageSource}
                    tabIndex={-1}>
                </img>
            </OverlayTrigger>
        );
        return indicatorNode;
    }

    /**
     * Selects the correct image based on 
     * @param result Result of the mission
     */
    private selectIndicatorImage(result?: MissionResult): string {
        switch(result) {
            case MissionResult.Pass:
                console.log("pass link here")
                return "https://i.imgur.com/pJTlA0g.png";
            case MissionResult.Fail:
                console.log("fail link here")
                return "https://i.imgur.com/pJTlA0g.png";
            default:
                console.log("greyscale link here")
                return "https://i.imgur.com/pJTlA0g.png";
        }
    }

    /**
     * Formats the players and cards played on a mission into readable English.
     * @param playersOnMission Player names of players on the mission.
     * @param playedCards Cards played on the mission.
     * @param result Result of the mission.
     */
    private initializePopover(missionNum: number, playersOnMission: string[], playedCards: Card[], result: MissionResult): JSX.Element {
        // First, add the players on the mission.
        let popoverPlayers = "Players: ";
        const numPlayers = playersOnMission.length;
        for (let i = 0; i < numPlayers; i++) {
            //Handle cases of the last player (and), a two player mission (no comma), or generic case.
            if (i + 1 === numPlayers) {
                popoverPlayers += "and " + playersOnMission[i] + " (" + numPlayers + ")";
            } else if (numPlayers === 2) {
                popoverPlayers += playersOnMission[i] + " ";
            } else {
                popoverPlayers += playersOnMission[i] + ", ";
            }
        }

        // Handle cards played.
        let popoverCardsPlayed = "Cards Played: ";
        const numCards = playedCards.length;
        for (let i = 0; i < numCards; i++) {
            popoverCardsPlayed += Card[playedCards[i]];
            if (i < numCards - 1) {
                popoverCardsPlayed += ", ";
            }
        }

        // Create the formatted JSX for the popover layout
        const popoverText = (
            <p>
                {popoverPlayers} 
                <br /> 
                {popoverCardsPlayed}
            </p>
        );
        
        // Initialize the popover and add information
        const title = `Mission ${missionNum} Summary`;
        const popover = (
            <Popover 
                title={title}
                id={"m" + missionNum + "Indicator"}
                placement="top">
                <Popover.Content>
                    {popoverText}
                </Popover.Content>
            </Popover>
        );
        return popover;
    }
    //#endregion
}

/**
 * Defines the props object for MissionIndicator.
 */
interface MissionIndicatorProps {
    missionNum: number,
    playersOnMission: string[],
    result: MissionResult,
    cardsPlayed: Card[]
}

interface MissionPlaceholderProps {
    missionNum: number,
    numPlayersOnMisison: number,
    requiresDoubleFail: boolean
}

/**
 * Shows mission information for missions that have not gone yet, such as player counts.
 */
export class MissionPlaceholderIndicator extends React.Component<MissionPlaceholderProps> {
    /**
     * Instantiates the placeholder and checks to ensure all required properties exist.
     * @param props Required properties to initialize the placeholder
     */
    constructor(props: MissionPlaceholderProps) {
        super(props);
        if (typeof props.missionNum !== "number"
            || typeof props.numPlayersOnMisison !== "number"
            || typeof props.requiresDoubleFail !== "boolean") {
            
            throw new MissingPropertyError("Mission number, number of players, and double fail are required.");
        }
    }

    /**
     * Renders the placeholder component.
     */
    render(): React.ReactNode {
        let doubleFailIndicator = <span></span>;
        if (this.props.requiresDoubleFail) {
            doubleFailIndicator = (
            <span className="circularText">
                <br />
                "Two Fails Required"
            </span>
        );}
        return (
            <p className="rounded-circle">
                <span className="numPlayerIndicator">
                    {this.props.missionNum}
                </span>
                <br />
                "Players"
                {doubleFailIndicator}
            </p>
        );
    }
} 

