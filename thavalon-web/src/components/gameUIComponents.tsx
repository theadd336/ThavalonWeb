import * as React from "react";
import { MissionIndicatorCollection } from "./missionIndicators";
import { RoleCaption } from "./roleInformation";
import { WebSocketManager, WebSocketProp } from "./communication";
import { MissingPropertyError } from "../Core/errors";
import { Nav, Navbar, Tabs, Tab, Container, Row, Col } from "react-bootstrap";
import { RoleInformationTab } from "./roleInformation";
import { VoteHistoryTab } from "./votingInformation";
import { PlayerOrderTab } from "./playerOrder";
import { ProposalVoteTab } from "./proposalVoting";


/**
 * Class for the center game board. Contains mission indicators and role blurb.
 * This high level class only passes websockets to children and maintains no state.
 */
export class GameBoard extends React.Component<WebSocketProp>
{
    /**
     * Instantiates the board and performs basic 
     * @param props Object containing the WebSocketManager instance.
     */
    constructor(props: WebSocketProp) {
        super(props);
        if (!(props.webSocket instanceof(WebSocketManager))) {
            throw new MissingPropertyError("There is no valid connection.");
        }
    }
    
    render(): JSX.Element {
        return (
            <Container className="pt-5">
                <Row>
                    <Col className="text-center" >
                        <RoleCaption webSocket={this.props.webSocket} />
                    </Col>
                </Row>
                <MissionIndicatorCollection webSocket={this.props.webSocket} />
            </Container>
        );
    }
}

/**
 * Constant class that represents the header.
 */
export class Header extends React.Component {
    render(): JSX.Element {
        return (
            <Navbar 
                bg="light"
                variant="light">
                <Navbar.Brand href="/thavalon">
                    Home
                </Navbar.Brand>
                <Nav className="mr-auto">
                    <Nav.Link href="/thavalon/ViewLobbies.html">
                        View Lobbies
                    </Nav.Link>
                </Nav>
                <Nav>
                    <Nav.Link href="#">
                        Rules
                    </Nav.Link>
                </Nav>
            </Navbar>
        );
    }
}

export class GameInformationCollection extends React.Component<WebSocketProp> {
    constructor(props: WebSocketProp) {
        super(props);
        if (!(props.webSocket instanceof WebSocketManager)) {
            throw new MissingPropertyError("The WebSocketManager is missing from the tabs collection.");
        }
    }

    render(): JSX.Element {
        return (
            <Container
                className="pt-3" 
                fluid>
                <Row>
                    <Col>
                        <GameTabCollection webSocket={this.props.webSocket} />
                    </Col>
                </Row>
            </Container>
        );
    }
}

class GameTabCollection extends React.Component<WebSocketProp> {
    constructor(props: WebSocketProp) {
        super(props);
        if (!(props.webSocket instanceof WebSocketManager)) {
            throw new MissingPropertyError("The WebSocketManager is missing from the tabs collection.");
        }
    }

    //#region Public Methods
    render(): JSX.Element {
        const webSocket = this.props.webSocket;
        return (
            <Tab.Container
                defaultActiveKey="roleInformation" 
                id="gameTabsCollection">
                
                <TabHeadersComponent />
                <Tab.Content>
                    <Tab.Pane
                        eventKey="roleInformation">
                        
                        <RoleInformationTab webSocket={webSocket} />
                    </Tab.Pane>
                    <Tab.Pane 
                        eventKey="voteHistory">
                        
                        <VoteHistoryTab webSocket={webSocket} />
                    </Tab.Pane>
                    <Tab.Pane
                        eventKey="proposalVoting">
                        
                        <ProposalVoteTab webSocket={webSocket} />
                    </Tab.Pane>
                    <Tab.Pane
                        eventKey="missionCards">

                    </Tab.Pane>
                    <Tab.Pane
                        eventKey="playerOrder">

                        <PlayerOrderTab webSocket={webSocket} />
                    </Tab.Pane>
                </Tab.Content>
            </Tab.Container>
        );
    }
    //#endregion
}

class TabHeadersComponent extends React.Component {
    render(): JSX.Element {
        return (
            <Nav
                variant="tabs"
                defaultActiveKey="roleInformation"
                role="tablist"
                id="gameTabsCollection"
                className="gameBoardTabs"
                fill>
                <Nav.Item>
                    <Nav.Link eventKey="roleInformation">
                        My Info
                    </Nav.Link>
                </Nav.Item>
                <Nav.Item>
                    <Nav.Link eventKey="voteHistory">
                        Voting History
                    </Nav.Link>
                </Nav.Item>
                <Nav.Item>
                    <Nav.Link eventKey="proposalVoting">
                        Proposals/Voting
                    </Nav.Link>
                </Nav.Item>
                <Nav.Item>
                    <Nav.Link eventKey="missionCards">
                        Mission
                    </Nav.Link>
                </Nav.Item>
                <Nav.Item>
                    <Nav.Link eventKey="playerOrder">
                        Player Order
                    </Nav.Link>
                </Nav.Item>
            </Nav>
        );
    }
}