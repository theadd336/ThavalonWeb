import React, {useState} from 'react';
import ReactModal from 'react-modal';
import { useForm } from 'react-hook-form';
import { Link, Redirect } from 'react-router-dom';
import { AccountManager } from '../utils/AccountManager';
import "./Login.scss";
import { InputElement } from './InputElement';

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
    const {register, handleSubmit, errors} = useForm<LoginData>();
    // state for setting if register button is disabled
    const [disable, setDisabled] = useState(false);
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
        setDisabled(true);

        const accountManager = AccountManager.getInstance();
        let httpResponse = await accountManager.loginUser(data.email, data.password);

        // on successful register, log in user to update navbar and redirect to home page.
        // On fail, set error message and re-enable register button
        if (httpResponse.result) {
            props.setLoggedIn(true);
            setRedirectToHome(true);
        } else {
            setFormErrorMsg(httpResponse.message);
            setDisabled(false);
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
                    <InputElement type="email" label="Email Address" required={true} minLength={0} />
                    <InputElement type="password" label="Password" required={true} minLength={8} />
                    <div className="formSubmission">
                        <input type="submit" disabled={disable} value="Log In" />
                        <span className="errorMsg">{formErrorMsg}</span>
                        <br />
                        <Link to="/register" className="formLink">Create Account?</Link>
                    </div>
                </form>
            </div>
        </ReactModal>
    );
}
