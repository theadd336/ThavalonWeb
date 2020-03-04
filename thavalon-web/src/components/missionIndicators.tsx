import * as React from "react";
import { Popover, OverlayTrigger, Card as BootstrapCard } from "react-bootstrap";
import { MissingPropertyError, InvalidMissionError, ConnectionError } from "../Core/errors";
import { MissionResult, Card, AllMissionInfo } from "../Core/gameConstants";
import FailToken from "../static/red-coin.png";
import SuccessToken from "../static/black-coin.png";
import { WebSocketProp, WebSocketManager } from "./communication";
import { WebSocketMessage, IncomingMessageTypes, MissionResultsMessage } from "../Core/commConstants";


//#region interfaces
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

/**
 * Defines the props object for MissionPlaceholderIndicator
 */
interface MissionPlaceholderProps {
    discriminator: "MissionPlaceholderProps",
    missionNum: number,
    numPlayersOnMisison: number,
    requiresDoubleFail: boolean
}

/**
 * Defines the props object for the MissionIndicatorCollection
 */
export interface AllMissionInfoMessage {
    numMissions: number,
    missionsInfo: (MissionIndicatorProps | MissionPlaceholderProps)[]
}

/**
 * Defines the state object for MissionIndicatorCollection
 */
interface MissionIndicatorCollectionState {
    missionsCollection: (MissionIndicatorProps | MissionPlaceholderProps)[]
}
//#endregion

/**
 * Collection of mission indicators. Maintains the list and handles initializing placeholders or indicators.
 */
export class MissionIndicatorCollection extends React.Component<WebSocketProp, MissionIndicatorCollectionState> {
    private _connection: WebSocketManager;
    /**
     * Instantiates a new collection of mission indicators.
     * @param props Properties for the collection. Includes either resulted mission info or placeholder info.
     */
    constructor(props: WebSocketProp) {
        super(props);
        if (!(props.webSocket instanceof(WebSocketManager))) {
            throw new MissingPropertyError("The connection manager is missing.");
        }

        // If all tests pass, initialize the state.
        this.state = {
            missionsCollection: []
        }

        this._connection = props.webSocket;
    }

    componentDidMount(): void {
        this._connection.onSuccessfulMessage.subscribe((sender, message) => {
            this.receiveSuccessfulMessage(sender, message);
        });
    }


    /**
     * Renders the collection of mission indicators.
     */
    render(): JSX.Element {
        const missionIndicators = this.state.missionsCollection.map((indicator) => {
            if (this.instanceOfMissionIndicatorProps(indicator)) {
                return (
                    <MissionIndicator 
                        discriminator={indicator.discriminator}
                        missionNum={indicator.missionNum}
                        playersOnMission={indicator.playersOnMission}
                        cardsPlayed={indicator.cardsPlayed}
                        result={indicator.result} />);
            } else {
                return (
                    <MissionPlaceholderIndicator
                        discriminator={indicator.discriminator}
                        missionNum={indicator.missionNum}
                        numPlayersOnMisison={indicator.numPlayersOnMisison}
                        requiresDoubleFail={indicator.requiresDoubleFail} />);
            }
        });
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
        const missionsCollection = this.state.missionsCollection;
        if (missionNum < 0 || missionNum >= missionsCollection.length) {
            throw new InvalidMissionError("The mission number does not exist.");
        }
        missionsCollection[missionNum] = missionResults;
        this.setState({missionsCollection: missionsCollection});
    }

    /**
     * Type guard to determine if the information provided is enough for a resulted mission.
     * @param object Object to type guard
     */
    private instanceOfMissionIndicatorProps(object: any): object is MissionIndicatorProps {
        return object.discriminator === "MissionIndicatorProps";
    }

    /**
     * Populates the entire mission collection on a connection or reconnection.
     * @param message Message object with information for all missions
     */
    private populateMissionCollection(message: WebSocketMessage) {
        //TODO: Figure out which message types we need for casting.
        const allMissionInfoMessage = message.data as AllMissionInfoMessage
        const missionCollection = [];
        const { numMissions, missionsInfo } = allMissionInfoMessage
        if (missionsInfo.length !== numMissions) {
            throw new InvalidMissionError("The number of missions to initialize must match the number of provided information objects.");
        }
        // For any information, validate which type it is. If the mission has actual information (in the case of a reconnect),
        // initialize a new indicator. Otherwise, initialize a placeholder. Then add it to the collection.
        for (const missionInfo of missionsInfo) {
            missionCollection.push(missionInfo);
        }
        // Post-instantiation validation.
        if (missionCollection.length !== numMissions) {
            throw new InvalidMissionError("Error during contruction of mission indicators.");
        }
        
        this.setState({missionsCollection: missionCollection})
    }

    /**
     * Handle a message from the server.
     * @param sender Object that sent this event. Currently unused.
     * @param message Message from the server.
     */
    private receiveSuccessfulMessage(sender: object, message: WebSocketMessage): void {
        // If it's not a mission result, we don't care.
        switch (message.type) {
            case IncomingMessageTypes.MissionResult:
                this.updateMissionResults(message);
                break;
            case IncomingMessageTypes.AllMissionInfo:
                this.populateMissionCollection(message);
                break;
            }
        }

    private updateMissionResults(message: WebSocketMessage) {
        // Grab the data and cast it appropriately. Also get the current collection of missions.
        const missionResult = message.data as MissionResultsMessage;
        const missionCollection = this.state.missionsCollection;
        // If the mission num is out of range, throw an error. This shouldn't happen.
        const missionNum = missionResult.priorMissionNum;
        if (missionNum < 0 || missionNum >= missionCollection.length) {
            throw new InvalidMissionError("The mission result is out of range.");
        }
        // Update the correct mission with the required information and set the state.
        missionCollection[missionNum] = {
            discriminator: "MissionIndicatorProps",
            missionNum: missionNum,
            playersOnMission: missionResult.playersOnMission,
            result: missionResult.missionResult,
            cardsPlayed: missionResult.playedCards
        };
        this.setState({ missionsCollection: missionCollection });
    }
}

//#region private classes
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
        return (
            <div className="col-2">
                {this.formatImageLink()}
            </div>
        );
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
                return SuccessToken;
            case MissionResult.Fail:
                return FailToken
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
        const missionCard = this.createCard();
        return (
            <div className="col-2">
                {missionCard}
            </div>);
    }

    /**
     * Creates a bootstrap card for the mission indicator.
     */
    private createCard(): JSX.Element {
        let doubleFailIndicator = "";
        if (this.props.requiresDoubleFail) {
            doubleFailIndicator = "Two Fails Required";
        }

        const card = (
            <BootstrapCard 
                bg="light" 
                className="rounded-circle indicatorPlaceholderStyle">
                <BootstrapCard.Body>
                    <BootstrapCard.Title>
                        {"Mission " + this.props.missionNum}
                    </BootstrapCard.Title>
                    <BootstrapCard.Text>
                        <span className="missionNumStyle">
                            {this.props.numPlayersOnMisison}
                        </span>
                        <br />
                        <span>
                            Players
                            <br />
                            <span className="text-danger">
                                {doubleFailIndicator}
                            </span>
                        </span>
                    </BootstrapCard.Text>
                </BootstrapCard.Body>
            </BootstrapCard>);

        return card;
    }
} 
//#endregion