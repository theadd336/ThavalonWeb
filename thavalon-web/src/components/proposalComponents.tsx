import React from "react";
import { Button, Form, ToggleButtonGroup, ToggleButton, Row } from "react-bootstrap";
import { WebSocketManager, WebSocketProp } from "./communication";
import { OutgoingMessageTypes } from "../Core/commConstants";

//#region Interfaces
interface ProposalSelectionFormProps {
    callback: (proposedPlayers: string[]) => any;
    playerOrder: string[];
    numOnProposal: number;
}

interface ProposalSelectionFormState {
    proposedPlayers: string[];
}

interface ProposalUIProps extends WebSocketProp {
    proposer: string;
    isProposing: boolean;
    proposal: string[];
    numOnProposal: number;
    playerOrder: string[];
}

interface TentativeProposalMessage {
    type: OutgoingMessageTypes.SubmitProposal;
    proposal: string[]
}

interface OutgoingMoveToVoteMessage {
    type: OutgoingMessageTypes.MoveToVote
    proposal: string[];
}
//#endregion

export class ProposalUI extends React.Component<ProposalUIProps, {proposal: string[]}> {
    private _connection: WebSocketManager;
    constructor(props: ProposalUIProps) {
        super(props);
        this._connection = props.webSocket;
        this.state = {proposal: []};
    }

    render(): JSX.Element {
        if (this.props.isProposing) {
            return this.createProposerUI();
        } else {
            return this.createOtherProposerUI();
        }
    }

    private createProposerUI(): JSX.Element {
        const currentProposal = this.formatOwnProposalList();
        return (
            <span>
                {currentProposal}
                <ProposalSelectionForm 
                    callback={(proposal: string[]) => {
                        this.sendTentativeProposal(proposal);
                    }}
                    numOnProposal={this.props.numOnProposal}
                    playerOrder={this.props.playerOrder} />
                <br />
                <Button 
                    type="button" 
                    onClick={this.moveToVote.bind(this)}>
                    Move to Vote
                </Button>
            </span>
        );
    }

    private formatOwnProposalList(): JSX.Element | null {
        if (this.state.proposal.length === 0) {
            return null;
        }
        const playerList = this.state.proposal.map((player) => {
            return (<li key={player}>{player}</li>);
        });
        const currentProposal = (
            <span>
                You have proposed: 
                <ul>{playerList}</ul>
            </span>
        );
        return currentProposal;
    }

    private createOtherProposerUI(): JSX.Element {
        const proposal = this.props.proposal;
        let proposalInfo: JSX.Element
        if (proposal.length === 0) {
            proposalInfo = (
                <span>
                    Please wait while {this.props.proposer} proposes a team.
                </span>
            );
        } else {
            proposalInfo = (
                <span>
                    {this.props.proposer} has proposed: 
                    {this.formatProposalList()}
                </span>
            );
        }
        return proposalInfo;
    }

    private formatProposalList(): JSX.Element {
        const proposal = this.props.proposal;
        const proposalList = proposal.map((player) => {
            return <li key={player}>{player}</li>
        });
        return (<ul>{proposalList}</ul>);
    }

    private sendTentativeProposal(proposal: string[]): void {
        const message: TentativeProposalMessage = {
            type: OutgoingMessageTypes.SubmitProposal,
            proposal: proposal
        };
        this._connection.send(message);
        this.setState({proposal: proposal});
    }

    private moveToVote(): void {
        if (this.state.proposal.length !== this.props.numOnProposal) {
            alert("Exactly " + this.props.numOnProposal + " players are required.");
            return;
        }
        const message: OutgoingMoveToVoteMessage = {
            type: OutgoingMessageTypes.MoveToVote,
            proposal: this.state.proposal
        };
        this._connection.send(message);
    }
}

class ProposalSelectionForm extends React.Component<ProposalSelectionFormProps, ProposalSelectionFormState> {
    constructor(props: ProposalSelectionFormProps) {
        super(props);
        this.state = {proposedPlayers: []}
    }

    render(): JSX.Element {
        const playerOptionsList = this.props.playerOrder.map((player) => {
            return (
                <ToggleButton
                    variant="outline-secondary" 
                    value={player}>
                    {player}
                </ToggleButton>
            );
        });
        return (
            <span>
                <Row>
                    <ToggleButtonGroup
                        vertical={true}
                        type="checkbox"
                        onChange={this.handleFormChange.bind(this)}>
                        {playerOptionsList}
                    </ToggleButtonGroup>
                </Row>
                <Row className="pt-4">
                    <Button 
                        type="button" 
                        onClick={this.handleSubmit.bind(this)}
                        variant="success">
                        Submit Proposal
                    </Button>
                </Row>
            </span>
        );
    }

    private handleSubmit(): void {
        if (this.state.proposedPlayers.length !== this.props.numOnProposal) {
            alert("Exactly " + this.props.numOnProposal + " players are required.");
            return;
        }
        this.props.callback(this.state.proposedPlayers);
    }

    private handleFormChange(currentProposedPlayers: string[]): void {
        this.setState({proposedPlayers: currentProposedPlayers});
    }

}