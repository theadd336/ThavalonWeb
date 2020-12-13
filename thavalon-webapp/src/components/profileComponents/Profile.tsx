import React from 'react';
import { Account } from './Account';
import { Stats } from './Stats';
import "../../styles/profileStyles/Profile.scss";

export function Profile(): JSX.Element {
    return (
        <div id="profileContainer">
            <Account />
            <Stats />
        </div>
    );
}