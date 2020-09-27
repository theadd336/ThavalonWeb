import { TabComponent } from "./tabComponents";
import { WebSocketProp } from "./communication";
import { GamePhase, Vote } from "../Core/gameConstants";
import React from "react";
import { OutgoingMessageTypes, IncomingMessageTypes, IncomingMessage, OutgoingMessage } from "../Core/commConstants";
import { ProposalUI } from "./proposalComponents";
import { VotingButtons } from "../Core/sharedComponents";
import { Container, Row, Col } from "react-bootstrap";
import { AbilityUI } from "./abilities/abilityComponents";


//#region Interfaces
interface ProposalVoteInfo {
    gamePhase: GamePhase;
    proposer: string;
    isProposing: boolean;
    proposal: string[];
    numOnProposal: number;
    proposalNumber: number;
    maxNumProposals: number;
    playerOrder: string[];
}

interface VoteUIProps extends WebSocketProp {
    proposal: string[];
}

interface VoteState {
    hasVoted: boolean;
    vote: Vote;
}

interface CommonInformationUIProps {
    isProposing: boolean;
    proposer: string;
    numOnProposal: number;
    proposalNumber: number;
    maxNumProposals: number;
}

interface SubmitVoteMessage {
    type: OutgoingMessageTypes.SubmitVote;
    vote: Vote
}

interface NewProposalMessage {
    proposer: string;
    isProposing: boolean;
    numOnProposal: number;
    proposalNumber: number;
    maxNumProposals: number;
}

interface ProposalReceivedMessage {
    proposal: string[];
}

interface PlayerOrderMessage {
    playerOrder: string[];
}

interface IncomingMoveToVoteMessage {
    proposal: string[];
}
//#endregion

export class ProposalVoteTab extends TabComponent<ProposalVoteInfo> {
    constructor(props: WebSocketProp) {
        super(props);
        this.state = {
            gamePhase: GamePhase.Proposal,
            proposer: "",
            isProposing: false,
            proposal: [],
            numOnProposal: 2,
            proposalNumber: 1,
            maxNumProposals: 3,
            playerOrder: []
        }
    }

    render(): JSX.Element {
        const gamePhase = this.state.gamePhase;
        let tab: JSX.Element;
        switch (gamePhase) {
            case GamePhase.Proposal:
                tab = (
                    <ProposalUI
                        webSocket={this.props.webSocket}
                        proposer={this.state.proposer}
                        isProposing={this.state.isProposing}
                        proposal={this.state.proposal}
                        numOnProposal={this.state.numOnProposal}
                        playerOrder={this.state.playerOrder}
                        maxNumProposals={this.state.maxNumProposals}
                        proposalNum={this.state.proposalNumber} />
                );
                break;
            case GamePhase.Voting:
                tab = (
                    <VoteUI
                        webSocket={this.props.webSocket}
                        proposal={this.state.proposal} />
                );
                break;
            default:
                tab = (
                    <span>Uh oh - you disconnected. Please wait for the next proposal.</span>
                );
                break;
        }
        return (
            <Container fluid={true}>
                <Row>
                    <Col>
                        <CommonInformationUI
                            isProposing={this.state.isProposing}
                            proposer={this.state.proposer}
                            numOnProposal={this.state.numOnProposal}
                            proposalNumber={this.state.proposalNumber}
                            maxNumProposals={this.state.maxNumProposals} />

                        {tab}
                    </Col>
                    <Col>
                        <AbilityUI
                            playerOrder={this.state.playerOrder}
                            webSocket={this.props.webSocket} />
                    </Col>
                </Row>
            </Container>
        );
    }

    protected receiveSuccessfulMessage(_: object, message: IncomingMessage): void {
        switch (message.type) {
            case IncomingMessageTypes.NewProposal:
                this.handleNewProposal(message.data as NewProposalMessage);
                break;
            case IncomingMessageTypes.MoveToVote:
                this.moveToVote(message.data as IncomingMoveToVoteMessage);
                break;
            case IncomingMessageTypes.PlayerOrder:
                this.setPlayerOrder(message.data as PlayerOrderMessage);
                break;
            case IncomingMessageTypes.ProposalReceived:
                this.receiveTentativeProposal(message.data as ProposalReceivedMessage);
                break;
        }
    }

    protected sendMessageOnMount(): OutgoingMessage {
        if (this.state.playerOrder.length === 0) {
            this.sendMessage({ type: OutgoingMessageTypes.PlayerOrder });
        }
        return { type: OutgoingMessageTypes.ProposalVoteInformationRequest };
    }

    private handleNewProposal(proposalData: NewProposalMessage): void {
        const newState = {
            gamePhase: GamePhase.Proposal,
            proposer: proposalData.proposer,
            isProposing: proposalData.isProposing,
            proposal: [],
            numOnProposal: proposalData.numOnProposal,
            proposalNumber: proposalData.proposalNumber,
            maxNumProposals: proposalData.maxNumProposals,
        }
        this.setState(newState);
    }

    private moveToVote(voteData: IncomingMoveToVoteMessage): void {
        const newState = {
            gamePhase: GamePhase.Voting,
            proposal: voteData.proposal
        }
        this.setState(newState);
    }

    private receiveTentativeProposal(proposalData: ProposalReceivedMessage): void {
        if (!this.state.isProposing) {
            this.setState(proposalData);
        }
    }

    private setPlayerOrder(playerOrderData: PlayerOrderMessage): void {
        this.setState(playerOrderData);
    }
}

//#region Private Classes
/**
 * Handles voting specific information, including the vote buttons and applicable event handlers.
 */
class VoteUI extends React.Component<VoteUIProps, VoteState> {
    /**
     * Initializes component's props and state.
     * @param props Props object containing proposal and web socket connection.
     */
    constructor(props: VoteUIProps) {
        super(props);
        this.state = {
            hasVoted: false,
            vote: Vote.Downvote
        }
    }

    /**
     * Renders the applicable vote state information. This will either be voting buttons or a wait message.
     */
    render(): JSX.Element {
        let voteContent: JSX.Element;
        if (this.state.hasVoted) {
            voteContent = this.renderAfterVote(this.state.vote);
        } else {
            voteContent = this.renderBeforeVote(this.props);
        }
        return voteContent;
    }

    /**
     * Event handler to submit a player's vote and update state accordingly.
     * @param submittedVote How the player voted.
     */
    onSubmitVote(submittedVote: Vote): void {
        const voteMessage: SubmitVoteMessage = {
            type: OutgoingMessageTypes.SubmitVote,
            vote: submittedVote
        }
        this.props.webSocket.send(voteMessage);
        const newState: VoteState = {
            vote: submittedVote,
            hasVoted: true
        }
        this.setState(newState);
    }

    //#region Private Methods
    /**
     * Creates a string indicating the player should wait for others to finish voting.
     * @param vote How the player voted.
     */
    private renderAfterVote(vote: Vote): JSX.Element {
        let afterVoteSentence = "You have ";
        if (vote === Vote.Upvote) {
            afterVoteSentence += "upvoted. ";
        } else {
            afterVoteSentence += "downvoted. ";
        }
        afterVoteSentence += "Please wait while others finish voting.";
        return (<p>{afterVoteSentence}</p>);
    }

    /**
     * Creates the UI prior to voting. This includes proposal information, the "Voting On" caption,
     * and the voting buttons.
     * @param proposalInformation All proposal information that the player will vote on.
     */
    private renderBeforeVote(proposalInformation: VoteUIProps): JSX.Element {
        const votingOn = "Voting On:";
        const playersOnProposal = this.createProposedPlayerList(proposalInformation.proposal);
        return (
            <p>
                {votingOn}
                {playersOnProposal}
                <VotingButtons callback={this.onSubmitVote.bind(this)} />
            </p>
        );
    }

    /**
     * Converts an array of players on the proposal to an unordered list.
     * @param playersOnProposal The players on the proposal
     */
    private createProposedPlayerList(playersOnProposal: string[]): JSX.Element {
        const proposedPlayerList = playersOnProposal.map((player) => {
            return <li key={player}>{player}</li>
        });
        return (<ul>{proposedPlayerList}</ul>);
    }
}

/**
 * Class representing information common to both proposals and voting.
 */
class CommonInformationUI extends React.Component<CommonInformationUIProps> {
    constructor(props: CommonInformationUIProps) {
        super(props);
    }

    /**
     * Renders the component in the DOM.
     */
    render(): JSX.Element {
        // Extract properties from the props object.
        const proposalNumber = this.props.proposalNumber;
        const maxNumProposals = this.props.maxNumProposals;
        const isProposing = this.props.isProposing;
        const numOnProposal = this.props.numOnProposal;
        const proposer = this.props.proposer;

        // Create the proposal number indicator and proposer sentence.
        const proposalNumberIndicator = this.createProposalNumberIndicator(proposalNumber, maxNumProposals);
        const proposerSentence = this.createProposerSentence(isProposing, numOnProposal, proposer);
        return (
            <p>
                {proposalNumberIndicator}
                <br />
                {proposerSentence}
            </p>
        );
    }

    /**
     * Creates the proposal number indicator with the force indicator if needed.
     * @param proposalNumber The number of the current proposal.
     * @param maxNumProposals The maximum number of proposals in the round.
     */
    private createProposalNumberIndicator(proposalNumber: number, maxNumProposals: number): JSX.Element {
        let proposalNumberIndicator = `Proposal ${ proposalNumber }/${ maxNumProposals } `;
        let forceIndicator: JSX.Element | null = null;
        if (proposalNumber === maxNumProposals) {
            forceIndicator = (
                <span className="text-danger">
                    [FORCE]
                </span>
            );
        }

        return (
            <span>
                {proposalNumberIndicator}
                {forceIndicator}
            </span>
        );
    }

    /**
     * Creates the sentence on whom is proposing the mission, with proper grammar.
     * @param isProposing True if the current player is proposing, false otherwise.
     * @param numOnMission Number of players on the proposal/mission.
     * @param proposer The name of the proposer. Unused if isProposing is true.
     */
    private createProposerSentence(isProposing: boolean, numOnMission: number, proposer?: string): string {
        let proposerSentence = "";
        if (isProposing) {
            proposerSentence += "You are ";
        } else {
            proposerSentence += proposer + " is ";
        }
        proposerSentence += "proposing a " + numOnMission + " player mission.";
        return proposerSentence;
    }
}

//#endregion