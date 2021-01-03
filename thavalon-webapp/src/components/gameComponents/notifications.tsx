import React, { useEffect, useState } from "react";
import { Toast } from "react-bootstrap";
import { GameSocket, InboundMessage, InboundMessageType } from "../../utils/GameSocket";
import { GameMessage, GameMessageType } from "./constants";
import "../../styles/gameStyles/notifications.scss";

enum ToastSeverity {
    INFO="info",
    WARN="warn",
    URGENT="urgent",
}

interface Toast {
    severity: ToastSeverity,
    message: string,
}

/**
 * Notifications panel of game page, for showing notices from server.
 */
export function Notifications(): JSX.Element {
    // the toasts currently being shown.
    const [toasts, setToasts] = useState<Toast[]>(new Array<Toast>());

    /**
     * Handle a game message.
     * @param gameMessage The game message.
     */
    function handleGameMessage(gameMessage: GameMessage): void {
        switch (gameMessage.messageType) {
            case GameMessageType.Toast: {
                const newArr = new Array(...toasts);
                newArr.push(gameMessage.data as Toast);
                setToasts(newArr);
                break;
            }
        }
    }

    /**
     * Handle an inbound message.
     * @param message The inbound message.
     */
    function handleMessage(message: InboundMessage): void {
        switch (message.messageType) {
            case InboundMessageType.GameMessage: {
                handleGameMessage(message.data as GameMessage);
                break;
            }
        }
    }

    // useEffect handles componentDidMount and componentWillUnmount steps.
    useEffect(() => {
        // On mount, get the connection instance and set up event handlers.
        // Then, get the player list.
        const connection = GameSocket.getInstance();
        connection.onGameEvent.subscribe(handleMessage);

        // On unmount, unsubscribe our event handlers.
        return () => {
            connection.onGameEvent.unsubscribe(handleMessage);
        }
    }, [toasts]);

    /**
     * Delete a toast from the shown toasts.
     * @param idx The index of the toast being removed.
     */
    function deleteToast(idx: number): void {
        const newArr = new Array(...toasts);
        newArr.splice(idx, 1); // remove the element in toasts at index idx
        setToasts(newArr);
    }

    return <>
            <h1 className="game-section-header">Notifications</h1>
            <div id="toastNotifications">
                {toasts.length === 0 && <span className="no-notifications">No notifications yet.</span>}
                {toasts.map((toast, idx) => 
                    <Toast className="toast-notification" onClose={() => deleteToast(idx)}>
                        <Toast.Header className={`toast-${toast.severity} toast-header`}>
                            <strong className="mr-auto">{toast.severity.toUpperCase()}</strong>
                        </Toast.Header>
                        <Toast.Body>
                            {toast.message}
                        </Toast.Body>
                    </Toast>
                )}
            </div>
    </>
}
