import React, {useEffect} from 'react';
import Modal from 'react-modal';
import { Redirect } from 'react-router-dom';
import { log_in } from '../utils/account_utils';

type LoginProps = {
    setLoggedIn: any
};
type LoginState = {};

Modal.setAppElement("#root");

function Login(props: LoginProps) {
    useEffect(() => props.setLoggedIn(log_in()));

    return (
        <Redirect to="/" />
    )
}

export default Login;