import React, { useState } from "react";
import { ButtonGroup, Button, Form, FormControl } from "react-bootstrap";
import { any } from "prop-types";

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
            return <option>{player}</option>
        });
        return (
            <Form 
                onSubmit={this.handleSubmit}>
                
                <Form.Group controlId="proposalSelectionForm">
                    <Form.Label>
                        Select {this.props.numOnProposal} Players
                    </Form.Label>
                    <Form.Control
                        onChange={this.handleFormChange}
                        ref="proposalSelectionFormControl"
                        as="select"
                        custom
                        multiple>

                        {playerOptionsList}
                    </Form.Control>
                </Form.Group>
                <Button type="submit">Submit Proposal</Button>
            </Form>
        )
    }

    private handleSubmit(event: React.FormEvent): void {
        this.props.callback(this.state.proposedPlayers);
    }

    private handleFormChange(event: any): void {
        const proposedPlayers = event.target.value;
        this.setState({proposedPlayers: proposedPlayers});
    }

}