import React, {useEffect, useState} from 'react';
import ReactModal from 'react-modal';
import { useForm } from 'react-hook-form';
import { Link, Redirect } from 'react-router-dom';
import styled from 'styled-components';
import { log_in } from '../utils/account_utils';
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

    function onError(data: any, event: any) {
        console.log(data);
        console.log("ERROR");
        event.preventDefault();
    }

    function onSubmit(data: any, event: any) {
        console.log("SUBMIT");
        console.log(data);
        event.preventDefault();
    }

    // log in without throwing a warning by using useEffect
    useEffect(() => props.setLoggedIn(log_in()));
    return (
        <ReactModal
            isOpen={modalIsOpen}
            onRequestClose={closeModal}
            contentLabel="Example Modal"
            className="Modal"
            overlayClassName="Overlay"
        >
            <h2>Log In</h2>
            <form onSubmit={handleSubmit(onSubmit, onError)}>
                <input
                    type="text"
                    placeholder="Email"
                    name="email"
                    ref={register({required: true, maxLength: 80, pattern: {
                        value: /^\S+@\S+\.\S+$/i,
                        message: "Invalid email address."
                    }})} />
                {errors.email && <span className="error_msg">{errors.email.message}</span>}
                <br />
                <input
                    type="text"
                    placeholder="Password"
                    name="password"
                    ref={register({required: true})} />
                {errors.password && <span className="error_msg">Password required.</span>}
                <br />
                <Link to="/register">Create Account?</Link>
                <br />
                <input type="submit" />
            </form>
        </ReactModal>
    )
}

export default Login;