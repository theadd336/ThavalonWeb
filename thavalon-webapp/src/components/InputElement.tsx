import React from 'react';
import "./InputElement.scss";

interface InputElementProps {
    "type": string,
    "label": string,
    "required": boolean,
    "minLength": number,
}

export function InputElement(props: InputElementProps) {
    const inputElement = React.useRef<HTMLInputElement>(null);
    const [inputClassName, setinputClassName] = React.useState("noContent");

    /**
     * Set the input element's class. Needed for scss selectors, because there's no way in css for determining
     * if box is empty or not. Cannot use valid/invalid because email inputs can be invalid even if input has
     * content.
     */
    function setLabelElementClass(): void {
        if (inputElement === null) {
            return;
        }

        const input = inputElement.current;
        if (input === null) {
            return;
        }

        if (input.value === "") {
            setinputClassName("noContent");
        } else {
            setinputClassName("content");
        }
    }

    return (
        <>
            <input type={props.type} required={props.required} onChange={setLabelElementClass} className={inputClassName} ref={inputElement} minLength={props.minLength} />
            <label placeholder={props.label} />
        </>
    );
};