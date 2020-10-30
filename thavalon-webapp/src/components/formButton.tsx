// This module contains code related to the FormButton componenet, used for
// submitting forms that might require loading.

import React, { useState } from 'react';
import './formButton.scss';

/**
 * Properties for the form button.
 * @param label  The label the button will use
 * @param isLoading  Whether or not the button should be in a loading state
 * @param color  The color of the button. Defaults to green.
 * @param size  The size of the button. Defaults to medium.
 */
export interface FormButtonProps {
    label: string
    isLoading: boolean
    color?: "red" | "green" | "grey"
    size?: "small" | "medium" | "large"
}

/**
 * The React component that renders the button and handles animations
 * @param props The properties used by the FormButton. 
 * @returns  The JSX.Element for the button.
 */
export function FormButton(props: FormButtonProps): JSX.Element {
    // Handle loading state. If we should be loading, set a timeout to add the 
    // `loading-spin` class which spins. 
    const [loadingSpin, setLoadingSpin] = useState("");
    if (props.isLoading === true && loadingSpin === "") {
        setTimeout(() => setLoadingSpin("loading-spin"), 150);
    }

    // Add in classes. Only add "active" and "loader" if isLoading is true, since
    // these cause animations.
    const classes = `form-button-${ props.color || "green" } ` + (props.isLoading ? `loading-start ${ loadingSpin }` : "");
    return (
        <div className={`button-container ${ props.size || "medium" }`}>
            <button
                type="submit"
                className={classes}
                disabled={props.isLoading}
            >
                {props.label}
            </button>
        </div>
    );
}
