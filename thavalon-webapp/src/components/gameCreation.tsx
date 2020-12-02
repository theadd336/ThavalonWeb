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

interface JoinGameData {
    friendCode: string,
    displayName: string
}

interface CreateGameData {
    displayName: string
}

export interface CreateJoinGameProps {
    show: boolean,
    onHide: Dispatch<SetStateAction<boolean>>
}

enum CreateJoinState {
    CreateJoinButtons,
    CreateGame,
    JoinGame
}

const connection = AccountManager.getInstance();

export function CreateJoinGameModal(props: CreateJoinGameProps): JSX.Element {
    const [showForm, setShowForm] = useState(false);
    const [modalState, setModalState] = useState(CreateJoinState.CreateJoinButtons);
    return (
        <ReactModal
            isOpen={props.show}
            onRequestClose={() => props.onHide(false)}
            contentLabel="Create/Join Game Modal"
            className="Modal"
            overlayClassName="Overlay">
            <div className="modalContainer">
                <h2 className="modalHeader">Play Thavalon</h2>
                <hr />
                {modalState === CreateJoinState.CreateJoinButtons && CreateJoinButtons(setShowForm, setModalState)}
                <CSSTransition
                    in={showForm}
                    timeout={300}
                    classNames="create-join-form"
                    onExited={() => setModalState(CreateJoinState.CreateJoinButtons)}
                    unmountOnExit>
                    <>
                        {modalState === CreateJoinState.CreateGame && <CreateGameForm setState={setShowForm} />}
                        {modalState === CreateJoinState.JoinGame && <JoinGameForm setState={setShowForm} />}
                    </>
                </CSSTransition>
            </div>
        </ReactModal >
    );
}

function CreateJoinButtons(showForm: any, setState: any): JSX.Element {
    return (
        <div className="create-join-buttons">
            <Container>
                <div style={{ textAlign: "center" }}>
                    <ButtonGroup vertical>
                        <Button
                            style={{ marginBottom: 10 }}
                            variant="primary"
                            onClick={() => {
                                showForm(true);
                                setState(CreateJoinState.JoinGame);
                            }}>
                            Join Game
                        </Button>
                        <Button
                            variant="primary"
                            onClick={() => {
                                showForm(true);
                                setState(CreateJoinState.CreateGame);
                            }}>
                            Create Game
                    </Button>
                    </ButtonGroup>

                </div>
            </Container>
        </div >
    );
}

function CreateGameForm(props: { setState: any }): JSX.Element {
    const { register, handleSubmit } = useForm<CreateGameData>();
    const [formErrorMsg, setFormErrorMsg] = useState("");
    const [redirectToGame, setRedirectToGame] = useState(false);
    const [friendCode, setFriendCode] = useState("");
    const [socketUrl, setSocketUrl] = useState("");

    async function onCreateGameSubmit(data: CreateGameData) {
        const createGameResponse = await connection.createGame();
        if (createGameResponse.result === false) {
            setFormErrorMsg(createGameResponse.message);
            return;
        }

        const friendCode = createGameResponse.message;
        const joinGameResponse = await connection.joinGame(friendCode, data.displayName);
        if (joinGameResponse.result === false) {
            setFormErrorMsg(joinGameResponse.message);
            return;
        }

        const socketUrl = joinGameResponse.message;
        setSocketUrl(socketUrl);
        setFriendCode(friendCode);
        setRedirectToGame(true);
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
                            onClick={() => props.setState(false)}>
                            Back
                        </Button>
                    </Col>
                    <Col>
                        <Button style={{ float: "right" }} type="submit" variant="primary">Create Game</Button>
                    </Col>
                </Row>
                <div className="errorMsg">
                    {formErrorMsg}
                </div>
            </form>
        </div >
    );
}

function JoinGameForm(props: { setState: any }): JSX.Element {
    const { register, handleSubmit } = useForm<JoinGameData>();
    const [formErrorMsg, setFormErrorMsg] = useState("");
    const [redirectToGame, setRedirectToGame] = useState(false);
    const [friendCode, setFriendCode] = useState("");
    const [socketUrl, setSocketUrl] = useState("");

    async function onJoinGameSubmit(data: JoinGameData): Promise<void> {
        const joinGameResponse = await connection.joinGame(data.friendCode, data.displayName);
        if (joinGameResponse.result === false) {
            setFormErrorMsg(joinGameResponse.message);
            return;
        }

        setFriendCode(data.friendCode);
        setSocketUrl(joinGameResponse.message);
        setRedirectToGame(true);
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
                            onClick={() => props.setState(false)}>
                            Back
                        </Button>
                    </Col>
                    <Col>
                        <Button style={{ float: "right" }} type="submit" variant="primary">Join Game</Button>
                    </Col>
                </Row>
                <div className="errorMsg">
                    {formErrorMsg}
                </div>
            </form>
        </div>
    );
}


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