import React, { useState } from 'react';
import ReactModal from 'react-modal';
import { InputElement } from './formComponents/InputElement';
import { useForm } from 'react-hook-form';
import { FormButton } from './formComponents/FormButton';

import "../styles/Modal.scss";
import "../styles/PlayGameModal.scss";

interface FriendCodeData {
    friendCode: string
}

export function CreateJoinGameModal() {
    // if set, register modal is open
    const [modalIsOpen, setModalIsOpen] = useState(true);
    const [showFriendCodeEntry, setShowFriendCodeEntry] = useState(true);
    const [formErrorMsg, setFormErrorMsg] = useState("");
    const { register, handleSubmit } = useForm<FriendCodeData>();

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
            <div className="modalContainer">
                <h2 className="modalHeader">Play</h2>
                <hr />
                <button onClick={() => setShowFriendCodeEntry(true)}>Join Game</button>
                <form>
                    <div className="join-game-horiz-fields">
                        <div>
                            <InputElement
                                formRef={register}
                                type="text"
                                label="Friend Code"
                                name="friendCode"
                                required={true}
                                minLength={4}
                                maxLength={4}
                                autoCapitalize={true}
                                autoComplete="off" />
                        </div>
                        <div className="join-game-submit-btn">
                            <FormButton label="Join" isLoading={false} color="green" size="small" />
                        </div>
                        <div className="errorMsg">{formErrorMsg}</div>
                    </div>
                </form>
                <button>Create Game</button>
            </div>
        </ReactModal >
    );
}