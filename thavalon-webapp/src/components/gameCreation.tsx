import React, { useState } from 'react';
import ReactModal from 'react-modal';
import { useForm } from 'react-hook-form';
import { Link, Redirect } from 'react-router-dom';
import { AccountManager } from '../utils/AccountManager';
import "./Modal.scss";


interface LoginData {
    "email": string,
    "password": string
};

export function CreateJoinGameModal() {
    // if set, register modal is open
    const [modalIsOpen, setModalIsOpen] = useState(true);

    /**
     * Called when register modal is closed.
     */
    function closeModal() {
        setModalIsOpen(false);
    }

    return (
        <ReactModal
            isOpen={modalIsOpen}
            onRequestClose={closeModal}
            contentLabel="Play Game Modal"
            className="Modal"
            overlayClassName="Overlay"
        >
            <h2 className="modalHeader">Log In</h2>
        </ReactModal>
    );
}
