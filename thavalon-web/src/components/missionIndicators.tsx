import * as React from "react";
import { Popover, OverlayTrigger } from "react-bootstrap";
import { MissingPropertyError, InvalidMissionError } from "../Core/errors";
import { MissionResult, Card, MAX_NUM_MISSIONS } from "../Core/gameConstants";
import { object } from "prop-types";

/**
 * Handles mission indicator images and popovers.
 */
class MissionIndicator extends React.Component<MissionIndicatorProps> {

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

    render(): JSX.Element {
        return this.formatImageLink();
    }

    //#region private methods
    /**
     * Formats the correct mission indicator and popover for display
     */
    private formatImageLink(): JSX.Element {
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
    discriminator: "MissionIndicatorProps",
    missionNum: number,
    playersOnMission: string[],
    result: MissionResult,
    cardsPlayed: Card[]
}

interface MissionPlaceholderProps {
    discriminator: "MissionPlaceholderProps",
    missionNum: number,
    numPlayersOnMisison: number,
    requiresDoubleFail: boolean
}

/**
 * Shows mission information for missions that have not gone yet, such as player counts.
 */
class MissionPlaceholderIndicator extends React.Component<MissionPlaceholderProps> {
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
    render(): JSX.Element {
        let doubleFailIndicator = <span></span>;
        if (this.props.requiresDoubleFail) {
            doubleFailIndicator = (
            <span className="circularText">
                <br />
                "Two Fails Required"
            </span>);
        }
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

/**
 * Collection of mission indicators. Maintains the list and handles initializing placeholders or indicators.
 */
export class MissionIndicatorCollection extends React.Component<{numMissions: number}, MissionIndicatorCollectionState> {
    /**
     * Instantiates a new collection of mission indicators.
     * @param props Properties for the collection. Includes either resulted mission info or placeholder info.
     */
    constructor(props: MissionIndicatorCollectionProps) {
        super({numMissions: props.numMissions});
        // Initialize variables and perform initial validation.
        const missionCollection = [];
        const allMissionInfo = props.missionsInfo;
        if (allMissionInfo.length !== this.props.numMissions) {
            throw new InvalidMissionError("The number of missions to initialize must match the number of provided information objects.");
        }

        // For any information, validate which type it is. If the mission has actual information (in the case of a reconnect),
        // initialize a new indicator. Otherwise, initialize a placeholder. Then add it to the collection.
        let missionIndicator: MissionPlaceholderIndicator | MissionIndicator
        for (let i = 0; i < props.numMissions; i++) {
            const missionInfo = allMissionInfo[i]; 
            if (this.instanceOfMissionIndicatorProps(missionInfo)) {
                missionIndicator = new MissionIndicator(missionInfo);
            } else {
                missionIndicator = new MissionPlaceholderIndicator(missionInfo);
            }
            missionCollection.push(missionIndicator);
        }

        // Post-instantiation validation.
        if (missionCollection.length !== props.numMissions) {
            throw new InvalidMissionError("Error during contruction of mission indicators.");
        }

        // If all tests pass, initialize the state.
        this.state = {
            missionsCollection: missionCollection
        }
    }

    /**
     * Renders the collection of mission indicators.
     */
    render(): JSX.Element {
        const missionIndicators = this.state.missionsCollection.map((indicator) =>
            <div className="col-2">
                {indicator}
            </div>
        );
        return (
            <div className="row">
                {missionIndicators}
            </div>
        );
    }

    /**
     * Updates the appropriate mission with the mission's results.
     * @param missionResults Results from the mission that now has results.
     */
    addMissionResults(missionResults: MissionIndicatorProps): void {
        if (typeof missionResults === undefined) {
            throw new MissingPropertyError("Mission results are required.");
        }
        const missionNum = missionResults.missionNum;
        if (missionNum < 0 || missionNum >= this.props.numMissions) {
            throw new InvalidMissionError("The mission number does not exist.");
        }
        const missionIndicator = new MissionIndicator(missionResults);
        const missionsCollection = this.state.missionsCollection;
        missionsCollection[missionNum] = missionIndicator;
        this.setState({missionsCollection: missionsCollection});
    }

    /**
     * Type guard to determine if the information provided is enough for a resulted mission.
     * @param object Object to type guard
     */
    private instanceOfMissionIndicatorProps(object: any): object is MissionIndicatorProps {
        return object.discriminator === "MissionIndicatorProps";
    }

}

interface MissionIndicatorCollectionProps {
    numMissions: number,
    missionsInfo: (MissionIndicatorProps | MissionPlaceholderProps)[]
}

interface MissionIndicatorCollectionState {
    missionsCollection: (MissionIndicator | MissionPlaceholderIndicator)[]
}