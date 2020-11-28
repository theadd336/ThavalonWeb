import React, { useState, Dispatch, SetStateAction } from 'react';
import ReactModal from 'react-modal';
import { InputElement } from './formComponents/InputElement';
import { useForm } from 'react-hook-form';
import { FormButton } from './formComponents/FormButton';
import { Container, Row, Col, Button, ButtonGroup, Modal } from "react-bootstrap";

import "../styles/Modal.scss";
import "../styles/PlayGameModal.scss";

interface FriendCodeData {
    friendCode: string
}

export interface CreateJoinGameProps {
    show: boolean,
    onHide: Dispatch<SetStateAction<boolean>>
}

export function CreateJoinGameModal(props: CreateJoinGameProps): JSX.Element {
    return (
        <Modal
            show={props.show}
            onHide={() => props.onHide(false)}
            centered
        >
            <Modal.Header closeButton>
                <Modal.Title>
                    Play Thavalon
                </Modal.Title>
            </Modal.Header>
            <Modal.Body>
                <Row>
                    <Button variant="primary">Join Existing Game</Button>
                </Row>
                <Row className="mt-2">
                    <Button variant="primary">Create New Game</Button>
                </Row>
            </Modal.Body>
        </Modal>
    );
}