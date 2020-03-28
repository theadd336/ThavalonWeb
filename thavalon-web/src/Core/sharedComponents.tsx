import React from "react";
import { Vote } from "./gameConstants";
import { ButtonGroup, Button } from "react-bootstrap";

interface VoteButtonProps {
    callback: (vote: Vote) => void;
    buttonStyle?: string;
}

/**
 * Class to render voting buttons. Use whenever you need the upvote and downvote buttons.
 */
export class VotingButtons extends React.Component<VoteButtonProps> {
    /**
     * Renders the buttons with applicable callbacks.
     */
    render(): JSX.Element {
        return (
            <ButtonGroup vertical>
                <Button
                    variant="primary"
                    onClick={() => this.props.callback(Vote.Upvote)}>
                    Upvote
                    </Button>
                <Button
                    variant="danger"
                    onClick={() => this.props.callback(Vote.Downvote)}>
                    Downvote
                    </Button>
            </ButtonGroup>
        );
    }
}
