import React, {useState} from 'react';
import ReactModal from 'react-modal';
import { DeepMap, FieldError, Resolver, useForm } from 'react-hook-form';
import { AccountManager, HttpResponse } from '../utils/AccountManager';
import { Redirect, useHistory } from 'react-router-dom';
import { InputElement } from './formComponents/InputElement';
import { FormButton } from './formComponents/FormButton';
import "../styles/Modal.scss";

interface RegisterProps {
    setLoggedIn: React.Dispatch<React.SetStateAction<boolean>>
    setShowLoginModal: React.Dispatch<React.SetStateAction<boolean>>
};

interface RegisterData {
    name: string,
    email: string,
    password: string,
    confirmPassword: string,
};

const registerResolver: Resolver<RegisterData> = async (values: RegisterData) => {
    return {
        values: values.email && values.name && values.password && values.confirmPassword ? values : {},
        errors: (values.password !== values.confirmPassword) ? {
                    confirmPassword: {
                        type: "required",
                        message: "Passwords do not match."
                    }
                } : {}
        }
    }

export function Register(props: RegisterProps): JSX.Element {
    // if set, register modal is open
    const [modalIsOpen, setModalIsOpen] = useState(true);
    // hook for register form
    const {register, handleSubmit} = useForm<RegisterData, Event>({
        resolver: registerResolver
    });
    // state for setting if form is being submitted or not
    const [formSubmitting, setFormSubmitting] = useState(false);
    // state for setting register error
    const [formErrorMsg, setFormErrorMsg] = useState("");
    // state for redirecting to home on successful login
    const [redirectToHome, setRedirectToHome] = useState(false);
    // react router history
    const history = useHistory();

    /**
     * Called when register modal is closed. Redirects to most recent non-modal page.
     */
    function closeModal() {
        setModalIsOpen(false);
        props.setShowLoginModal(false);
        // For register, prior page (-1) is where login was triggered, because login is not its own page
        // TODO: if user goes directly to register page then closes modal, this will take them off site currently
        history.go(-1);
    }

    /**
     * Function for handling errors from the resolver. Currently if called, only error should
     * have the key confirmPassword.
     * @param data The errors caused by the resolver.
     */
    function onError(data: DeepMap<RegisterData, FieldError>) {
        if (data.confirmPassword === undefined || data.confirmPassword.message === undefined) {
            console.log("Form onError called with no errors.");
            return;
        }
        setFormErrorMsg(data.confirmPassword.message);
    }

    /**
     * On error, just prevent page reload - form handles showing errors.
     * @param data The data being sent on submit.
     * @param event The event caused by submission.
     */
    async function onSubmit(data: RegisterData) {
        // disable button on start of submit
        setFormSubmitting(true);
        setFormErrorMsg("");

        // attempt registering of user
        const accountManager: AccountManager = AccountManager.getInstance();
        let httpResponse: HttpResponse = await accountManager.registerUser(data.name, data.email, data.password);

        // on successful register, log in user to update navbar and redirect to home page.
        // On fail, set error message and re-enable register button
        if (httpResponse.result) {
            props.setLoggedIn(true);
            setRedirectToHome(true);
        } else {
            setFormErrorMsg(httpResponse.message);
            setFormSubmitting(false);
        }
        props.setShowLoginModal(false);
    }

    if (redirectToHome) {
        return <Redirect to="/" />;
    }

    return (
        <ReactModal
            isOpen={modalIsOpen}
            onRequestClose={closeModal}
            contentLabel="Register Modal"
            className="Modal"
            overlayClassName="Overlay"
        >
            <div className="modalContainer">
                <h2 className="modalHeader">Register</h2>
                <hr />
                <form onSubmit={handleSubmit(onSubmit, onError)}>
                    <InputElement formRef={register} type="text" label="Name" name="name" required={true} />
                    <InputElement formRef={register} type="email" label="Email Address" name="email" required={true} />
                    <InputElement formRef={register} type="password" label="Password" name="password" required={true} minLength={8} />
                    <InputElement formRef={register} type="password" label="Confirm Password" name="confirmPassword" required={true} minLength={8} />
                    <div className="formSubmission">
                        <FormButton label="Register" isLoading={formSubmitting} color="green" size="large" />
                        <span className="errorMsg">{formErrorMsg}</span>
                    </div>
                </form>
            </div>
        </ReactModal>
    );
}