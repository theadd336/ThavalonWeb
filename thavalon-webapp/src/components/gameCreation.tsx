import React, { useState, Dispatch, SetStateAction } from 'react';
import ReactModal from 'react-modal';
import { InputElement } from './formComponents/InputElement';
import { useForm } from 'react-hook-form';
import { Container, Row, Col, Button, ButtonGroup, Modal } from "react-bootstrap";
import { CSSTransition } from 'react-transition-group';

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

export function CreateJoinGameModal(props: CreateJoinGameProps): JSX.Element {
    async function OnSubmit(): Promise<void> { }
    const [showForm, setShowForm] = useState(false);
    const [modalState, setModalState] = useState(CreateJoinState.CreateJoinButtons);
    return (
        <ReactModal
            isOpen={props.show}
            onRequestClose={() => props.onHide(false)}
            contentLabel="Create/Join Game Modal"
            className="Modal"
            overlayClassName="Overlay"
        >
            <div className="modalContainer">
                <h2 className="modalHeader">Play Thavalon</h2>
                <hr />
                {modalState === CreateJoinState.CreateJoinButtons && renderCreateJoinButtons(setShowForm, setModalState)}
                <CSSTransition
                    in={showForm}
                    timeout={300}
                    classNames="create-join-form"
                    onExited={() => setModalState(CreateJoinState.CreateJoinButtons)}
                    unmountOnExit>
                    <>
                        {modalState === CreateJoinState.CreateGame && <CreateGameForm setState={setShowForm} onSubmitCallback={OnSubmit} />}
                        {modalState === CreateJoinState.JoinGame && <JoinGameForm setState={setShowForm} onSubmitCallback={OnSubmit} />}
                    </>
                </CSSTransition>
            </div>
        </ReactModal >
    );
}

function renderCreateJoinButtons(showForm: any, setState: any): JSX.Element {
    return (
        <div className="create-join-buttons">
            <Container>
                <Row>
                    <Button
                        variant="primary"
                        onClick={() => {
                            showForm(true);
                            setState(CreateJoinState.JoinGame);
                        }}>
                        Join Game
                    </Button>
                </Row>
                <Row>
                    <Button
                        variant="primary"
                        onClick={() => {
                            showForm(true);
                            setState(CreateJoinState.CreateGame);
                        }}>
                        Create Game
                    </Button>
                </Row>
            </Container>
        </div>
    );
}

function CreateGameForm(props: { setState: any, onSubmitCallback: any }): JSX.Element {
    const { register, handleSubmit } = useForm<CreateGameData>();
    return (
        <div className="create-game-form">
            <form onSubmit={handleSubmit(props.onSubmitCallback)}>
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
            </form>
        </div>
    );
}

function JoinGameForm(props: { setState: any, onSubmitCallback: any }): JSX.Element {
    const { register, handleSubmit } = useForm<JoinGameData>();
    return (
        <div className="join-game-form">
            <form onSubmit={handleSubmit(props.onSubmitCallback)}>
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
            </form>
        </div>
    );
}