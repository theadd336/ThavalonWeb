import React, { useEffect } from 'react';
import { Redirect } from 'react-router-dom';
import { log_out } from '../utils/account_utils';

type LogoutProps = {
    setLoggedIn: any
};

function Logout(props: LogoutProps) {
    useEffect(() => props.setLoggedIn(!log_out()));

    return (
        <Redirect to="/" />
    )
}


export default Logout;