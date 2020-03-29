import React from "react";
import { WebSocketProp, WebSocketManager } from "../communication";
import { OutgoingMessageTypes, IncomingMessageTypes, IncomingMessage } from "../../Core/commConstants";
import { Col, Button, Container, Row, ToggleButton, ToggleButtonGroup } from "react-bootstrap";
import { Vote } from "../../Core/gameConstants";
import { VotingButtons } from "../../Core/sharedComponents";

//#region interfaces

/**
 * Props object for the AbilityUI component.
 */
interface AbilityUIProps extends WebSocketProp {
    playerOrder: string[];
}

/**
 * State object for the AbilityUI component.
 */
interface AbilityUIState {
    description: string;
    caption: string;
    canUseAbility: boolean;
    hasUsedAbility: boolean;
    needsPlayerList: boolean;
    needsVoteOptions: boolean;
    abilityTimeout?: number;
}

/**
 * Incoming ability information message from the server.
 */
interface AbilityInformationMessage {
    description: string;
    caption: string;
    canUseAbility: boolean;
    needsPlayerList: boolean;
    needsVoteOptions: boolean;
    abilityTimeout?: number;
}

interface AdditionalAbilityProps {
    caption: string;
    playerOrder: string[];
    needsPlayerList: boolean;
    needsVoteOptions: boolean;
    callback: (player?: string, vote?: Vote) => void;
}
//#endregion

/**
 * Class representing the UI for abilities. Handles description, submitting, and 
 * timeouts, if needed.
 */
export class AbilityUI extends React.Component<AbilityUIProps, AbilityUIState> {
    private _connection: WebSocketManager;
    private _timerStart: number | null;
    private _timerID: NodeJS.Timeout | null;
    /**
     * Instantiates the class and sets applicable state.
     * @param props Object containing the active web socket connection and player order.
     */
    constructor(props: AbilityUIProps) {
        super(props);
        this.state = {
            description: "",
            caption: "Use Ability",
            canUseAbility: false,
            hasUsedAbility: false,
            needsPlayerList: false,
            needsVoteOptions: false
        }

        this._connection = this.props.webSocket;
        this._timerStart = null;
        this._timerID = null;
    }

    /**
     * Renders the component with applicable information.
     */
    render(): JSX.Element | null {
        const { description, caption, canUseAbility, hasUsedAbility, abilityTimeout, needsVoteOptions, needsPlayerList } = this.state;
        let additionalInformation = "";
        let additionalComponent: JSX.Element

        if (canUseAbility === false) {
            return null;
        }

        if (hasUsedAbility === true) {
            additionalComponent = <span>You have used your ability.</span>;
        } else {
            additionalComponent = (
                <AdditionalAbilityComponent
                    caption={caption}
                    playerOrder={this.props.playerOrder}
                    needsPlayerList={needsPlayerList}
                    needsVoteOptions={needsVoteOptions}
                    callback={this.useAbility.bind(this)} />
            );
        }
        if (typeof abilityTimeout === "number") {
            additionalInformation = this.renderTimedAbilityStatement(abilityTimeout);
        }

        return (
            <Col>
                {description}
                <br />
                {additionalInformation}
                <br />
                {additionalComponent}
            </Col>
        );
    }

    /**
     * Sets up event handlers for abilities and sends the ability request.
     */
    componentWillMount(): void {
        this._connection.onAbilityTypeMessage.subscribe((sender, message) => {
            this.receiveAbilityMessage(sender, message)
        });
        this._connection.send({ type: OutgoingMessageTypes.AbilityInformationRequest });
    }

    /**
     * Unsubscribes from appicable events.
     */
    componentWillUnmount(): void {
        this._connection.onAbilityTypeMessage.unsubscribe((sender, message) => {
            this.receiveAbilityMessage(sender, message)
        });
    }

    /**
     * Receives an ability message from the server and sets state.
     * @param _ Unused
     * @param message Message from the server with ability information.
     */
    private async receiveAbilityMessage(_: object, message: IncomingMessage): Promise<void> {
        if (message.type !== IncomingMessageTypes.AbilityInformationResponse) { return; }
        const messageData = message.data as AbilityInformationMessage;
        if (typeof messageData.abilityTimeout === "number") {
            this._timerStart = Date.now();
            this._timerID = setInterval(
                () => this.tick(),
                500
            );
        }
        this.setState(messageData);
    }

    /**
     * Renders the timeout statement and keeps track of time.
     * @param abilityTimeout Number of seconds until timeout occurs.
     */
    private renderTimedAbilityStatement(abilityTimeout: number): string {
        return "You have " + abilityTimeout + " seconds to use your ability.";
    }

    /**
     * Handles each tick of the timer. This method is currently a mess.
     */
    private tick(): void {
        //#TODO: Clean all of this up.
        const timeRemaining = this.state.abilityTimeout;
        if (this._timerID === null) {
            return;
        }

        if (timeRemaining === undefined || timeRemaining <= 0 || this._timerStart === null) {
            clearInterval(this._timerID);
            const newState = {
                canUseAbility: false,
                abilityTimeout: undefined
            }
            this.setState(newState);
            return;
        }

        const delta = Math.floor((Date.now() - this._timerStart) / 1000);
        if (delta === timeRemaining) { return; }
        const newState = {
            abilityTimeout: delta
        }
        this.setState(newState);
    }

    /**
     * Sends a message to the server to send the ability.
     * @param player The player the ability is acting on.
     * @param vote The new vote for the player.
     */
    private useAbility(player?: string, vote?: Vote): void {
        const message = {
            type: OutgoingMessageTypes.UseAbility,
            player: player,
            vote: vote
        }
        this._connection.send(message);
        this.setState({ hasUsedAbility: true, abilityTimeout: undefined });
    }
}

/**
 * Class handling any additional information required for ability use.
 */
class AdditionalAbilityComponent extends React.Component<AdditionalAbilityProps, { player?: string, vote?: Vote }> {
    constructor(props: AdditionalAbilityProps) {
        super(props)
        this.state = {
            player: undefined,
            vote: undefined
        }
    }
    render(): JSX.Element {
        let playerList: JSX.Element | null = null;
        let voteButtons: JSX.Element | null = null;
        if (this.props.needsPlayerList === true) {
            playerList = this.createPlayerList(this.props.playerOrder);
        }
        if (this.props.needsVoteOptions === true) {
            voteButtons = <VotingButtons callback={this.handleVoteFormChange.bind(this)} />;
        }

        return (
            <Container fluid>
                <Row>
                    <Col>
                        {playerList}
                    </Col>
                    <Col className="pl-5">
                        {voteButtons}
                    </Col>
                </Row>
                <Row className="pt-5">
                    <Button
                        type="button"
                        onClick={() => this.props.callback(this.state.player, this.state.vote)}>
                        {this.props.caption}
                    </Button>
                </Row>
            </Container>
        );
    }

    private createPlayerList(playerOrder: string[]): JSX.Element {
        const playerOptionsList = playerOrder.map((player) => {
            return (
                <ToggleButton
                    variant="outline-secondary"
                    value={player}>
                    {player}
                </ToggleButton>
            );
        });
        return (
            <ToggleButtonGroup
                vertical={true}
                type="radio"
                name="playersToTargetWithAbility"
                onChange={this.handlePlayerFormChange.bind(this)}>
                {playerOptionsList}
            </ToggleButtonGroup>
        );
    }

    private handlePlayerFormChange(player?: string): void {
        this.setState({ player: player });
    }

    private handleVoteFormChange(vote?: Vote): void {
        this.setState({ vote: vote });
    }
}