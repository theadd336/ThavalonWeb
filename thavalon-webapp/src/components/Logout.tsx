import React, { useEffect } from 'react';
import { Redirect } from 'react-router-dom';
import { AccountManager, HttpResponse } from '../utils/AccountManager';

interface LogoutProps {
    setLoggedIn: React.Dispatch<React.SetStateAction<boolean>>
};

export function Logout(props: LogoutProps): JSX.Element {
    useEffect(() => {
        const accountManager = AccountManager.getInstance();
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
    );
}
