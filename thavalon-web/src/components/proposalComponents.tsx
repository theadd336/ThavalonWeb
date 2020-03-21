import React from "react";
import { Button, Form, ToggleButtonGroup, ToggleButton } from "react-bootstrap";

//#region Interfaces
interface ProposalSelectionFormProps {
    callback: (proposedPlayers: string[]) => any;
    playerOrder: string[];
    numOnProposal: number;
}

interface ProposalSelectionFormState {
    proposedPlayers: string[];
}
//#endregion

export class ProposalSelectionForm extends React.Component<ProposalSelectionFormProps, ProposalSelectionFormState> {
    constructor(props: ProposalSelectionFormProps) {
        super(props);
        this.state = {proposedPlayers: []}
    }

    render(): JSX.Element {
        const playerOptionsList = this.props.playerOrder.map((player) => {
            return <ToggleButton value={player}>{player}</ToggleButton>;
        });
        return (
            <span>
                <ToggleButtonGroup
                    vertical={true}
                    type="checkbox"
                    onChange={this.handleFormChange.bind(this)}>
                    {playerOptionsList}
                </ToggleButtonGroup>
                <br />
                <br />
                <Button type="button" onClick={this.handleSubmit.bind(this)}>
                    Submit Proposal
                </Button>
            </span>
        );
    }

    private handleSubmit(): void {
        if (this.state.proposedPlayers.length > this.props.numOnProposal) {
            alert("Only " + this.props.numOnProposal + " players are allowed.");
            return;
        }
        this.props.callback(this.state.proposedPlayers);
    }

    private handleFormChange(currentProposedPlayers: string[]): void {
        this.setState({proposedPlayers: currentProposedPlayers});
    }

}