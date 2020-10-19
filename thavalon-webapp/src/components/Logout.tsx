import React, { useEffect } from 'react';
import { Redirect } from 'react-router-dom';
import AccountManager, { HttpResponse } from '../utils/accountManager';

type LogoutProps = {
    setLoggedIn: any
};

function Logout(props: LogoutProps) {
    useEffect(() => {
        const accountManager: AccountManager = AccountManager.getInstance();
        accountManager.logoutUser().then((httpResponse: HttpResponse) => {
            if (httpResponse.result) {
                props.setLoggedIn(false);
            } else {
                console.log("Failed to log out: " + httpResponse.message);
            }
        });
    });

    return (
        <Redirect to="/" />
    )
}


export default Logout;