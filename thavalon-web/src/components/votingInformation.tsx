import React from "react"
import { TabComponent } from "./tabComponents";
import { Vote } from "../Core/gameConstants";
import { WebSocketProp } from "./communication";
import { IncomingMessage, IncomingMessageTypes } from "../Core/commConstants";
import { Tab, Col, Row, ListGroup } from "react-bootstrap";

//#region Private Interfaces
/**
 * Represents the state object of the VoteHistoryTab
 */
interface VoteHistoryState {
    voteHistory: VoteResultMessage[]
}

/**
 * Represents the VoteResultMessage received from the server.
 */
interface VoteResultMessage {
    missionNumber: number
    proposalNumber: number
    voteInformation: {
        [key: string]: Vote
    }
}
//#endregion

/**
 * Maintains a tab of all vote history throughout the game.
 */
export class VoteHistoryTab extends TabComponent<VoteHistoryState> {
    /**
     * Initializes the vote history tab.
     * @param props Props object containing the web socket manager.
     */
    constructor(props: WebSocketProp) {
        super(props);
        this.state = {
            voteHistory: []
        }
    }

    /**
     * Renders the tab will all available vote history.
     */
    render(): JSX.Element {
        const voteHistoryArray = this.state.voteHistory;
        const voteHistoryList = this.createVoteHistoryList(voteHistoryArray);
        const voteHistoryInfo = this.createVoteHistoryInfo(voteHistoryArray);
        return (
            <Tab.Container 
                id="vote-history"
                defaultActiveKey={"#link" + voteHistoryArray.length}>
                <Row>
                    <Col sm={4}>
                        {voteHistoryList}
                    </Col>
                    <Col sm={8}>
                        {voteHistoryInfo}
                    </Col>
                </Row>
            </Tab.Container>
        );
    }

    //#region Private Methods
    /**
     * Creates the list buttons for the vote history tab.
     * @param voteHistory An array of all vote results that have occurred.
     */
    private createVoteHistoryList(voteHistory: VoteResultMessage[]): JSX.Element {
        const voteHistoryList = [];
        let count = 0;
        for (const voteResult of voteHistory) {
            count++;
            voteHistoryList.push(
                <ListGroup.Item 
                    action 
                    href={"#link" + count}>
                    {`Mission ${voteResult.missionNumber + 1}: Proposal ${voteResult.proposalNumber + 1}`}
                </ListGroup.Item>
            );
        }
        return (
            <ListGroup>
                {voteHistoryList}
            </ListGroup>
        );
    }

    /**
     * Creates the actual info section for the vote history tab.
     * @param voteHistory An array of all vote results that have occurred.
     */
    private createVoteHistoryInfo(voteHistory: VoteResultMessage[]): JSX.Element {
        const voteHistoryInfo = [];
        let count = 0;
        for (const voteResult of voteHistory) {
            count++;
            const playersAndVotes = this.voteResultsToNamesAndVotes(voteResult.voteInformation);
            voteHistoryInfo.push(
                <Tab.Pane eventKey={"#link" + count}>
                    {playersAndVotes}
                </Tab.Pane>
            );
        }
        return (
            <Tab.Content>
                {voteHistoryInfo}
            </Tab.Content>
        );
    }

    /**
     * Creates a string array of the form "PlayerName: Vote"
     * @param voteInformation Object with player names as keys and their votes as values.
     */
    private voteResultsToNamesAndVotes(voteInformation: {[key: string]: Vote}): JSX.Element[] {
        const playersAndVotes = [];
        let voteString = "";
        for (const playerName in voteInformation) {
            const vote = voteInformation[playerName];
            // Convert the vote enum to a string.
            if (vote === Vote.Upvote) {
                voteString = "upvoted";
            } else {
                voteString = "downvoted";
            }
            // Add voting and player information to the array.
            playersAndVotes.push(
                <p>{playerName + ": " + voteString}</p>
            );
        }
        return playersAndVotes;
    }

    /**
     * Receives a successful message from the server and updates vote history if needed.
     * @param _ Unusued.
     * @param message Incoming message from the server with voting information.
     */
    protected receiveSuccessfulMessage(_: object, message: IncomingMessage): void {
        if (message.type !== IncomingMessageTypes.VoteResult) {
            return;
        }

        const data = message.data as VoteResultMessage;
        const currentHistory = this.state.voteHistory;
        currentHistory.push(data);
        this.setState({voteHistory: currentHistory});
    }
    //#endregion
}

export class VotingTab extends TabComponent {
    constructor(props: WebSocketProp) {
        super(props);
    }
    
}