import * as React from "react";
import { MissionIndicatorCollection } from "./missionIndicators";
import { RoleCaption } from "./roleInformation";
import { WebSocketManager, WebSocketProp } from "./communication";
import { MissingPropertyError } from "../Core/errors";
import { Nav, Navbar } from "react-bootstrap";


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
            <div className="container pt-5">
                <RoleCaption webSocket={this.props.webSocket} />
                <MissionIndicatorCollection webSocket={this.props.webSocket} />
            </div>
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