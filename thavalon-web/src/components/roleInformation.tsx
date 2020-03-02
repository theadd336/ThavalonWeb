import * as React from "react";
import { Team } from "../Core/gameConstants";
import { MissingPropertyError, ConnectionError } from "../Core/errors";
import { WebSocketManager, WebSocketProp } from "./communication";
import { WebSocketMessage, RoleInformationMessage, OutgoingMessageTypes, IncomingMessageTypes } from "../Core/commConstants";


interface RoleCaptionState {
    role: string,
    team: Team
}


/**
 * Component to track and format the role blurb above the board.
 */
export class RoleCaption extends React.Component<WebSocketProp, RoleCaptionState>
{
    private _connection: WebSocketManager;
    /**
     * Initializes state and confirms a WS manager exists.
     * @param props Prop object with the websocket manager.
     */
    constructor(props: WebSocketProp) {
        super(props);
        if (!(props.webSocket instanceof(WebSocketManager))) {
            throw new MissingPropertyError("Connection manager missing.");
        }
        this.state = {
            role: "",
            team: Team.Good
        }
        this._connection = props.webSocket;
    }
    //#region Public Methods
    /**
     * Sets up event handlers when the component is loaded into the DOM.
     */
    componentDidMount(): void {
        if (!this._connection.IsOpen) {
            throw new ConnectionError();
        }

        this._connection.onSuccessfulMessage.subscribe((sender, message) => {
            this.messageReceived(sender, message);
        })

        this._connection.send({"type": OutgoingMessageTypes.RoleInformation});
    }

    /**
     * Cleans up event handlers when this instance is destroyed.
     */
    componentWillUnmount(): void {
        this._connection.onSuccessfulMessage.unsubscribe((sender, message) => {
            this.messageReceived(sender, message);
        });
    }

    /**
     * Renders the component with the role and team.
     */
    render(): JSX.Element {
        let teamIndicator: JSX.Element;
        if (this.state.team === Team.Evil) {
            teamIndicator = <span className="text-danger"> [EVIL]</span>;
        } else {
            teamIndicator = <span className="text-success"> [GOOD]</span>;
        }
        return (
            <div className="row col-12 text-center">
                <span>
                    {"You are " + this.state.role} 
                    {teamIndicator}
                </span>
            </div>
        );
    }
    //#endregion
    //#region Private Methods
    private messageReceived(sender: object, message: WebSocketMessage): void {
        if (message.type !== IncomingMessageTypes.RoleInformation) {
            return;
        }

        const data = message.data as RoleInformationMessage;
        if (typeof data.role === "undefined"
            || typeof data.team === "undefined") {
            
            throw new MissingPropertyError("Role and Team are required.");
        }
        const state = {role: data.role, team: data.team};
        this.setState(state);
    }
    //#endregion
}