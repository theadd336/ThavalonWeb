import React, {useState} from 'react';
import ReactModal from 'react-modal';
import { useForm } from 'react-hook-form';
import { Link, Redirect } from 'react-router-dom';
import { AccountManager } from '../utils/AccountManager';
import "./Modal.scss";

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
            <h2 className="modalHeader">Log In</h2>
            <form onSubmit={handleSubmit(OnSubmit)}>
                <input
                    type="email"
                    placeholder="Email"
                    name="email"
                    ref={register({required: true, maxLength: 80, pattern: {
                        value: /^\S+@\S+\.\S+$/i,
                        message: "Invalid email address."
                    }})} />
                {errors.email && <span className="errorMsg">{errors.email.message}</span>}
                <br />
                <input
                    type="password"
                    placeholder="Password"
                    name="password"
                    ref={register({required: true, minLength: 8})} />
                {errors.password && <span className="errorMsg">Password required.</span>}
                <br />
                <Link to="/register" className="formLink">Create Account?</Link>
                <br />
                <input type="submit" disabled={disable} value="Log In" />
                <span className="errorMsg">{formErrorMsg}</span>
            </form>
        </ReactModal>
    );
}
