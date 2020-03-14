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

export class GameTabCollection extends React.Component<WebSocketProp> {
    constructor(props: WebSocketProp) {
        super(props);
        if (!(props.webSocket instanceof WebSocketManager)) {
            throw new MissingPropertyError("The WebSocketManager is missing from the tabs collection.");
        }
    }

    //#region Public Methods
    render(): JSX.Element {
        return <span></span>
    }
    //#endregion

    //#region Private Methods
    /**
     * Creates a primary tab to render. Primary tabs do have focus on start. 
     * There should only be on primary tab.
     * @param id HTML ID of the tab. Used by CSS.
     * @param children Any JSX children that need to be rendered.
     */
    private createPrimaryTab(id: string, ...children: JSX.Element[]): JSX.Element | null {
        if (typeof id !== "string") {
            return null;
        }

        const primaryTab = (
            <div 
                className="tab-pane fade show active"
                id={id}
                role="tabpanel"
                aria-labelledby={id + "-tab"}>
                
                {children}
            </div>
        );
        return primaryTab;
    }

    /**
     * Creates a secondary tab to render. Secondary tabs do not have focus on start.
     * @param id HTML ID of the tab. Used by CSS.
     * @param children Any JSX children that need to be rendered.
     */
    private createSecondaryTab(id: string, ...children: JSX.Element[]): JSX.Element | null {
        if (typeof id !== "string") {
            return null;
        }

        const secondaryTab = (
            <div 
                className="tab-pane fade"
                id={id}
                role="tabpanel"
                aria-labelledby={id + "-tab"}>
                
                {children}
            </div>
        );
        return secondaryTab;
    }
    //#endregion
}