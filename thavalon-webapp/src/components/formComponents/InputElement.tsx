import React from 'react';
import "../../styles/formStyles/InputElement.scss";

export interface InputElementProps {
    type: string,
    label: string,
    name: string,
    required: boolean,
    minLength?: number,
    formRef: any, // not fully clear what the type of register from react-hook-form is
}

/**
 * A input component, with appropriate styling.
 * @param props The props for the input element.
 */
export function InputElement(props: InputElementProps): JSX.Element {
    const inputElement = React.useRef<HTMLInputElement>(null);
    const [inputClassName, setInputClassName] = React.useState("noContent");

    /**
     * Set the input element's class. Needed for scss selectors, because there's no way in css for determining
     * if box is empty or not. Cannot use valid/invalid because email inputs can be invalid even if input has
     * content.
     */
    function setLabelElementClass(): void {
        if (inputElement?.current?.value === "") {
            setInputClassName("noContent");
        } else {
            setInputClassName("content");
        }
    }

    return (
        <>
            <input
                ref={props.formRef}
                type={props.type}
                required={props.required}
                onChange={setLabelElementClass}
                name={props.name}
                className={`${ inputClassName } ${ props.autoCapitalize ? "force-uppercase" : "" }`}
                minLength={props.minLength}
                maxLength={props.maxLength}
                autoComplete={props.autoComplete}
            />
            <label placeholder={props.label} />
        </>
    );
};