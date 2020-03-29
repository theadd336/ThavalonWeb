import React from "react";
import { Vote } from "./gameConstants";
import { ButtonGroup, Button, Toast } from "react-bootstrap";
import { WebSocketProp, WebSocketManager } from "../components/communication";
import { IncomingMessage, IncomingMessageTypes } from "./commConstants";

interface VoteButtonProps {
    callback: (vote: Vote) => void;
    buttonStyle?: string;
}

interface ToastNotificationState {
    currentToasts: JSX.Element[]
}

interface ToastMessage {
    message: string;
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

/**
 * Toast notification manager class that handles toasts from the server.
 */
export class ToastNotification extends React.Component<WebSocketProp, ToastNotificationState> {
    private _toastID: number;
    private _connection: WebSocketManager;
    constructor(props: WebSocketProp) {
        super(props);
        this.state = { currentToasts: [] }
        this._toastID = 0;
        this._connection = this.props.webSocket;
    }

    render(): JSX.Element {
        return (
            <>
                {this.state.currentToasts}
            </>
        );
    }

    componentWillMount(): void {
        this._connection.onToastNotificationMessage.subscribe((sender, message) => this.receiveToastMessage(sender, message));
    }

    componentWillUnmount(): void {
        this._connection.onToastNotificationMessage.unsubscribe((sender, message) => this.receiveToastMessage(sender, message));
    }

    private async receiveToastMessage(_: object, message: IncomingMessage): Promise<void> {
        if (message.type !== IncomingMessageTypes.ToastNotification) {
            return;
        }
        const toastData = message.data as ToastMessage;
        this.createNewToast(toastData);
    }

    private async createNewToast(toastData: ToastMessage): Promise<void> {
        const uniqueID = this._toastID;
        const toast = (
            <Toast
                key={this._toastID}
                show={true}
                onClose={() => this.removeToastFromState(uniqueID)}>
                <Toast.Header>
                    <strong>Ability Used!</strong>
                </Toast.Header>
                <Toast.Body>
                    {toastData.message}
                </Toast.Body>
            </Toast>
        );
        this._toastID++;
        const currentToasts = this.state.currentToasts;
        currentToasts.unshift(toast);
        this.setState({ currentToasts: currentToasts });
    }

    private removeToastFromState(toastIDToRemove: number): void {
        const currentToasts = [];
        for (const toast of this.state.currentToasts) {
            if (toast.key !== toastIDToRemove) {
                currentToasts.push(toast);
            }
        }
        this.setState({ currentToasts: currentToasts });
    }
}