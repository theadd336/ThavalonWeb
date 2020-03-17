import * as React from "react";
import { Team } from "../Core/gameConstants";
import { MissingPropertyError } from "../Core/errors";
import { WebSocketManager, WebSocketProp } from "./communication";
import { IncomingMessage, OutgoingMessageTypes, IncomingMessageTypes } from "../Core/commConstants";
import { TabComponent } from "./tabComponents";


//#region Private interfaces
interface RoleInformationState {
    role: string,
    team: Team
    description?: string
}

interface RoleInformationMessage {
    role: string,
    team: Team,
    description: string
}
//#endregion

//#region Public Classes
/**
 * Component to track and format the role blurb above the board.
 */
export class RoleCaption extends React.Component<WebSocketProp, RoleInformationState>
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
            <span>
                {"You are " + this.state.role} 
                {teamIndicator}
            </span>
        );
    }
    //#endregion
    //#region Private Methods
    private messageReceived(sender: object, message: IncomingMessage): void {
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

/**
 * Represents the role information tab and updates whenever any new role information is available.
 */
export class RoleInformationTab extends TabComponent<RoleInformationState> {
    /**
     * Initializes the role information tab and sets initial state.
     * @param props The active web socket connection.
     */
    constructor(props: WebSocketProp) {
        super(props);
        this.state = {
            role: "",
            team: Team.Good,
            description: ""
        };
    }

    /**
     * Handles receiving a successful message from the server.
     * @param _ Unused
     * @param message The incoming message with role information.
     */
    protected receiveSuccessfulMessage(_: object, message: IncomingMessage): void {
        if (message.type !== IncomingMessageTypes.RoleInformation) {
            return;
        }
        const data = message.data as RoleInformationMessage;
        const newState = {
            role: data.role,
            team: data.team,
            description: data.description
        }
        this.setState(newState);
    }

    /**
     * Renders the role information tab with information.
     */
    render(): JSX.Element {
        let formattedInfoString = "-------------------------\r\n";
        formattedInfoString += this.state.description + "\r\n";
        formattedInfoString += "-------------------------\r\n";
        return (
            <pre>
                {formattedInfoString}
            </pre>
        );
    }
}
//#endregion