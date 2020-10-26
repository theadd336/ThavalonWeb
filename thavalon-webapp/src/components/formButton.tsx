import React, { useState, useRef } from 'react';
import { CSSTransition, SwitchTransition } from 'react-transition-group';
import './formButton.scss';

export interface FormButtonProps {
    label: string
    isLoading: boolean
}

export function FormButton(props: FormButtonProps): JSX.Element {
    const formButtonRef = useRef<HTMLButtonElement>(null);
    const formButton = (
        <button
            type="submit"
            ref={formButtonRef}
        >
            {props.label}
        </button>
    );

    if (props.isLoading === true) {
        changeToLoading(formButtonRef.current);
    } else {
        changeToButton(formButtonRef.current);
    }

    return (
        <div className="inner-body">
            {formButton}
        </div>
    );
}

function changeToLoading(formButtonRef: HTMLButtonElement | null): void {
    if (formButtonRef === null) { return; }
    formButtonRef.classList.add("active");
    setTimeout(() => {
        formButtonRef.classList.add("loader");
    }, 125);
}

function changeToButton(formButtonRef: HTMLButtonElement | null): void {
    if (formButtonRef === null) return;
    formButtonRef.classList.remove("active", "loader")
}
