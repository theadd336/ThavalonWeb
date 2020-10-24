import React, {useState} from 'react';
import ReactModal from 'react-modal';
import { useForm } from 'react-hook-form';
import "./Modal.scss";
import { AccountManager, HttpResponse } from '../utils/accountManager';
import { Redirect } from 'react-router-dom';

type RegisterProps = {
    setLoggedIn: React.Dispatch<React.SetStateAction<boolean>>
};

interface RegisterData {
    "name": string,
    "email": string,
    "password": string,
    "confirmPassword": string,
};

export function Register(props: RegisterProps) {
    // if set, register modal is open
    const [modalIsOpen, setModalIsOpen] = useState(true);
    // hook for register form
    const {register, handleSubmit, errors} = useForm<RegisterData, Event>();
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
    async function onSubmit(data: RegisterData) {
        // disable button on start of submit
        // TODO: Also add loading image
        setDisabled(true);
        setFormErrorMsg("");

        // confirm passwords match. If they do not, show error message.
        if (data.password !== data.confirmPassword) {
            setFormErrorMsg("Passwords do not match");
            setDisabled(false); 
            return;
        }

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
            setDisabled(false);
        }
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
            <h2 className="modalHeader">Register</h2>
            <form onSubmit={handleSubmit(onSubmit)}>
                <input
                    type="text"
                    placeholder="Name"
                    name="name"
                    ref={register({required: true})}
                    />
                {errors.name && <span className="errorMsg">Name required.</span>}
                <br />
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
                {errors.password && <span className="errorMsg">Invalid Password.</span>}
                <br />
                <input
                    type="password"
                    placeholder="Confirm Password"
                    name="confirmPassword"
                    ref={register({required: true, minLength: 8})} />
                {errors.confirmPassword && <span className="errorMsg">Invalid Password.</span>}
                <br />
                <input type="submit" disabled={disable} value="Register" />
                <span className="errorMsg">{formErrorMsg}</span>
            </form>
        </ReactModal>
    );
}