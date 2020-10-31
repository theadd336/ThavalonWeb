import React, {useState} from 'react';
import ReactModal from 'react-modal';
import { useForm } from 'react-hook-form';
import { Link, Redirect } from 'react-router-dom';
import { AccountManager } from '../utils/AccountManager';
import { InputElement } from './formComponents/InputElement';
import { FormButton } from './formComponents/FormButton';
import "../styles/Modal.scss";

interface LoginProps {
    setLoggedIn: React.Dispatch<React.SetStateAction<boolean>>
};

interface LoginData {
    "email": string,
    "password": string
};

export function Login(props: LoginProps) {
    // if set, register modal is open
    const [modalIsOpen, setModalIsOpen] = useState(true);
    // hook for register form
    const {register, handleSubmit} = useForm<LoginData>();
    // state for setting if form is being submitted or not
    const [formSubmitting, setFormSubmitting] = useState(false);
    // state for setting register error
    const [formErrorMsg, setFormErrorMsg] = useState("");
    // state for redirecting to home on successful login
    const [redirectToHome, setRedirectToHome] = useState(false);

    /**
     * Called when register modal is closed.
     */
    function closeModal() {
        // TODO: Redirect to prior page on modal close so can refresh
        setModalIsOpen(false);
    }

    /**
     * On error, just prevent page reload - form handles showing errors.
     * @param data The data being sent on submit.
     * @param event The event caused by submission.
     */
    async function OnSubmit(data: LoginData) {
        console.log(data);
        setFormSubmitting(true);
        const accountManager = AccountManager.getInstance();
        let httpResponse = await accountManager.loginUser(data.email, data.password);

        // on successful register, log in user to update navbar and redirect to home page.
        // On fail, set error message and re-enable register button
        if (httpResponse.result) {
            props.setLoggedIn(true);
            setRedirectToHome(true);
        } else {
            setFormErrorMsg(httpResponse.message);
            setFormSubmitting(false);
        }
    }

    if (redirectToHome) {
        return <Redirect to="/" />;
    }

    // log in without throwing a warning by using useEffect

    return (
        <ReactModal
            isOpen={modalIsOpen}
            onRequestClose={closeModal}
            contentLabel="Login Modal"
            className="Modal"
            overlayClassName="Overlay"
        >
            <div className="modalContainer">
                <h2 className="modalHeader">Log In</h2>
                <hr />
                <form onSubmit={handleSubmit(OnSubmit)}>
                    <InputElement formRef={register} type="email" label="Email Address" name="email" required={true} minLength={0} />
                    <InputElement formRef={register} type="password" label="Password" name="password" required={true} minLength={8} />
                    <div className="formSubmission">
                        <FormButton label="Log In" isLoading={formSubmitting} color="green" size="large" />
                        <div className="errorMsg">{formErrorMsg}</div>
                        <Link to="/register" className="formLink">Create Account</Link>
                    </div>
                </form>
            </div>
        </ReactModal>
    );
}
