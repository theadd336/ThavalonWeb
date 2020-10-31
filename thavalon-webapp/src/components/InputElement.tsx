import React from 'react';
import { propTypes } from 'react-bootstrap/esm/Image';
import "./InputElement.scss";

export interface InputElementProps {
    "type": string,
    "label": string,
    "name": string,
    "required": boolean,
    "minLength": number,
    "formRef": any, // not fully clear what the type of register from react-hook-form is
}

/**
 * A input component, with appropriate styling.
 * @param props The props for the input element.
 */
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
            <input ref={props.formRef} type={props.type} required={props.required} onChange={setLabelElementClass} name={props.name} className={inputClassName} minLength={props.minLength} />
            <label placeholder={props.label} />
        </>
    );
};