import React, { useState, Dispatch, SetStateAction } from 'react';
import ReactModal from 'react-modal';
import { InputElement } from './formComponents/InputElement';
import { useForm } from 'react-hook-form';
import { Container, Row, Col, Button, ButtonGroup } from "react-bootstrap";
import { AccountManager } from "../utils/AccountManager";
import { CSSTransition } from "react-transition-group";
import { Redirect } from "react-router-dom";

import "../styles/Modal.scss";
import "../styles/PlayGameModal.scss";
import { GameSocket } from '../utils/GameSocket';

/**
 * Props for the CreateJoinGame Modal
 */
export interface CreateJoinGameProps {
    show: boolean,
    setOpen: Dispatch<SetStateAction<boolean>>
}

/**
 * Required data for the joinGame call.
 */
interface JoinGameData {
    friendCode: string,
    displayName: string
}

/**
 * Required data to create a game.
 */
interface CreateGameData {
    displayName: string
}

/**
 * Props for both the Join and Create Game forms
 */
interface FormProps {
    showForm: Dispatch<SetStateAction<boolean>>,
    onSuccess: () => void
}

/**
 * Enum representing the current modal state. Used to switch between the different forms.
 */
enum CreateJoinState {
    CreateJoinButtons,
    CreateGame,
    JoinGame
}

const connection = AccountManager.getInstance();

/**
 * Creates the modal for the join and create game options. This modal appears
 * when a user clicks "Play" and can either be used to create join a game.
 * @param props - Required React props object.
 */
export function CreateJoinGameModal(props: CreateJoinGameProps): JSX.Element {
    // State to keep track of whether or not the form should display.
    // Needed to trigger closing animations when the form is leaving the modal.
    const [showForm, setShowForm] = useState(false);
    // State to track the overall modal state, including which form should be displayed.
    const [modalState, setModalState] = useState(CreateJoinState.CreateJoinButtons);

    /**
     * Function handler for the success event from the child modal.
     * Will reset state for future callers and close the modal.
     */
    function onSuccess(): void {
        setModalState(CreateJoinState.CreateJoinButtons);
        props.setOpen(false);
    }

    return (
        <ReactModal
            isOpen={props.show}
            onRequestClose={() => props.setOpen(false)}
            contentLabel="Create/Join Game Modal"
            className="Modal"
            overlayClassName="Overlay">
            <div className="modalContainer">
                <h2 className="modalHeader">Play THavalon</h2>
                <hr />
                {modalState === CreateJoinState.CreateJoinButtons && CreateJoinButtons(setShowForm, setModalState)}
                <CSSTransition
                    in={showForm}
                    timeout={300}
                    classNames="create-join-form"
                    onExited={() => setModalState(CreateJoinState.CreateJoinButtons)}
                    unmountOnExit>
                    <>
                        {modalState === CreateJoinState.CreateGame && <CreateGameForm showForm={setShowForm} onSuccess={onSuccess} />}
                        {modalState === CreateJoinState.JoinGame && <JoinGameForm showForm={setShowForm} onSuccess={onSuccess} />}
                    </>
                </CSSTransition>
            </div>
        </ReactModal>
    );
}

/**
 * 
 * @param showForm Sets whether or not a form should display
 * @param setModalState Sets the modal state to the correct form that should display
 */
function CreateJoinButtons(showForm: Dispatch<SetStateAction<boolean>>, setModalState: Dispatch<SetStateAction<CreateJoinState>>): JSX.Element {
    return (
        <div className="create-join-buttons">
            <Container>
                <div className="center">
                    <ButtonGroup vertical>
                        <Button
                            className="with-bottom-margin"
                            variant="primary"
                            onClick={() => {
                                showForm(true);
                                setModalState(CreateJoinState.JoinGame);
                            }}>
                            Join Game
                        </Button>
                        <Button
                            variant="primary"
                            onClick={() => {
                                showForm(true);
                                setModalState(CreateJoinState.CreateGame);
                            }}>
                            Create Game
                        </Button>
                    </ButtonGroup>
                </div>
            </Container>
        </div >
    );
}

/**
 * Creates the "Create Game" form and handles submissions.
 * @param props Props to set the form state and handle successes
 */
function CreateGameForm(props: FormProps): JSX.Element {
    // Form to submit CreateGameData
    const { register, handleSubmit } = useForm<CreateGameData>();
    // State that tracks the form error message.
    const [formErrorMsg, setFormErrorMsg] = useState("");
    // State to check if we should redirect to a game.
    const [redirectToGame, setRedirectToGame] = useState(false);
    // State to maintain the friend code to send to the redirect link.
    const [friendCode, setFriendCode] = useState("");
    // State to maintain the socketUrl to send to the lobby component.
    const [socketUrl, setSocketUrl] = useState("");

    /**
     * Handles a submission to create a game by the player. Will create the game
     * and then auto-join the game, if there are no errors.
     * @param data The CreateGameData from the submitting form
     */
    async function onCreateGameSubmit(data: CreateGameData) {
        if (friendCode === "") {
            const createGameResponse = await connection.createGame();
            if (createGameResponse.result === false) {
                setFormErrorMsg(createGameResponse.message);
                return;
            }
            const friendCode = createGameResponse.message;
            setFriendCode(friendCode);
        }

        const joinGameResponse = await connection.joinGame(friendCode, data.displayName);
        if (joinGameResponse.result === false) {
            setFormErrorMsg(joinGameResponse.message);
            return;
        }

        const socketUrl = joinGameResponse.message;
        // create the gamesocket here, after join game succeeds
        GameSocket.createInstance(socketUrl);
        setSocketUrl(socketUrl);
        setRedirectToGame(true);
        props.showForm(false);
        props.onSuccess();
        return;
    }

    if (redirectToGame) {
        return triggerRedirectToGame(friendCode, socketUrl);
    }

    return (
        <div className="create-game-form">
            <form onSubmit={handleSubmit(onCreateGameSubmit)}>
                <InputElement
                    formRef={register}
                    type="text"
                    label="Display Name"
                    name="displayName"
                    required={true} />
                <Row>
                    <Col>
                        <Button
                            variant="secondary"
                            onClick={() => props.showForm(false)}>
                            Back
                        </Button>
                    </Col>
                    <Col>
                        <Button
                            className="submit-form-button"
                            type="submit"
                            variant="primary">
                            Create Game
                        </Button>
                    </Col>
                </Row>
                <div className="errorMsg">
                    {formErrorMsg}
                </div>
            </form>
        </div >
    );
}

/**
 * Creates the "Join Game" form and handles submissions.
 * @param props Props to set the form state and handle successes
 */
function JoinGameForm(props: FormProps): JSX.Element {
    // Form to submit CreateGameData
    const { register, handleSubmit } = useForm<JoinGameData>();
    // State that tracks the form error message.
    const [formErrorMsg, setFormErrorMsg] = useState("");
    // State to check if we should redirect to a game.
    const [redirectToGame, setRedirectToGame] = useState(false);
    // State to maintain the friend code to send to the redirect link.
    const [friendCode, setFriendCode] = useState("");
    // State to maintain the socketUrl to send to the lobby component.
    const [socketUrl, setSocketUrl] = useState("");

    /**
     * Handles a submission to join a game by the player. 
     * @param data The JoinGameData from the submitting form
     */
    async function onJoinGameSubmit(data: JoinGameData): Promise<void> {
        const joinGameResponse = await connection.joinGame(data.friendCode, data.displayName);
        if (joinGameResponse.result === false) {
            setFormErrorMsg(joinGameResponse.message);
            return;
        }

        setFriendCode(data.friendCode);
        setSocketUrl(joinGameResponse.message);
        setRedirectToGame(true);
        props.showForm(false);
        props.onSuccess();
        return;
    }

    if (redirectToGame) {
        return triggerRedirectToGame(friendCode, socketUrl);
    }

    return (
        <div className="join-game-form">
            <form onSubmit={handleSubmit(onJoinGameSubmit)}>
                <InputElement
                    formRef={register}
                    type="text"
                    label="Display Name"
                    name="displayName"
                    required={true}
                    maxLength={20} />
                <InputElement
                    autoComplete="off"
                    formRef={register}
                    type="text"
                    label="Friend Code"
                    name="friendCode"
                    required={true}
                    minLength={4}
                    maxLength={4} />
                <Row>
                    <Col>
                        <Button
                            variant="secondary"
                            onClick={() => props.showForm(false)}>
                            Back
                        </Button>
                    </Col>
                    <Col>
                        <Button
                            className="submit-form-button"
                            type="submit"
                            variant="primary">
                            Join Game
                        </Button>
                    </Col>
                </Row>
                <div className="errorMsg">
                    {formErrorMsg}
                </div>
            </form>
        </div>
    );
}

/**
 * Handles rendering the "component" that redirects to the game, setting up
 * necessary game parameters.
 * @param friendCode The friend code of the game to join. Used to create the link.
 * @param socketUrl The URL of the websocket. Needed by the lobby component to connect.
 */
function triggerRedirectToGame(friendCode: string, socketUrl: string): JSX.Element {
    if (friendCode === "" || socketUrl === "") {
        // Something is horribly broken if we're redirecting to a game with
        // no friend code or socketUrl. Log and bail out.
        console.log("ERROR: Tried to redirect to a game with no friend code or socket URL.");
        return (
            <Redirect to="/" />
        );
    }

    return (
        <Redirect to={{
            pathname: `/game/${ friendCode }`,
            state: { socketUrl: socketUrl }
        }} />
    );
}