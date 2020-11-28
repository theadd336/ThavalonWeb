import React, { useState } from 'react';
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

export function CreateJoinGameModal(props: { show: boolean }) {
    return (
        <Modal
            size="lg"
            show={props.show}
            centered
        >
            <Modal.Header closeButton>
                <Modal.Title>
                    Play Thavalon
                </Modal.Title>
            </Modal.Header>
            <Modal.Body>
                <Button variant="primary">Join Existing Game</Button>
                <Button variant="primary">Create New Game</Button>
            </Modal.Body>
        </Modal>
    );
}