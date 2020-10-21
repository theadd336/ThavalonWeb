import React, {useEffect, useState} from 'react';
import ReactModal from 'react-modal';
import { useForm } from 'react-hook-form';
import { Link, Redirect } from 'react-router-dom';
import { log_in, say_hello } from '../utils/account_utils';

import "./modal.scss";
type LoginProps = {
    setLoggedIn: any
};

ReactModal.setAppElement("#root");

function Login(props: LoginProps) {
    const [modalIsOpen, setModalIsOpen] = useState(true);
    const {register, handleSubmit, errors} = useForm();
    function closeModal() {
        setModalIsOpen(false);
    }

    function OnError(data: any, event: any) {
        console.log(data);
        console.log("ERROR");

        // prevent page from reloading
        event.preventDefault();
    }

    function OnSubmit(data: any, event: any) {
        console.log("SUBMIT");
        console.log(data);

        // try logging in, if it works update page to reflect that
        let loggedIn = log_in();
        useEffect(() => props.setLoggedIn(log_in()));

        // prevent page from reloading
        event.preventDefault();
    }

    // log in without throwing a warning by using useEffect
    return (
        <ReactModal
            isOpen={modalIsOpen}
            onRequestClose={closeModal}
            contentLabel="Login Modal"
            className="Modal"
            overlayClassName="Overlay"
        >
            <h2 className="modalHeader">Log In</h2>
            <form onSubmit={handleSubmit(OnSubmit, OnError)}>
                <input
                    type="text"
                    placeholder="Email"
                    name="email"
                    ref={register({required: true, maxLength: 80, pattern: {
                        value: /^\S+@\S+\.\S+$/i,
                        message: "Invalid email address."
                    }})} />
                {errors.email && <span className="errorMsg">{errors.email.message}</span>}
                <br />
                <input
                    type="text"
                    placeholder="Password"
                    name="password"
                    ref={register({required: true})} />
                {errors.password && <span className="errorMsg">Password required.</span>}
                <br />
                <Link to="/register" className="formLink">Create Account?</Link>
                <br />
                <input type="submit" value="Log In" />
            </form>
        </ReactModal>
    )
}

export default Login;