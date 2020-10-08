import React, {useEffect, useState} from 'react';
import ReactModal from 'react-modal';
import { useForm } from 'react-hook-form';
import "./modal.scss";
import { register_user } from '../utils/account_utils';

ReactModal.setAppElement("#root");

function Login() {
    const [modalIsOpen, setModalIsOpen] = useState(true);
    const {register, handleSubmit, errors} = useForm();
    function closeModal() {
        setModalIsOpen(false);
    }

    function onError(data: any, event: any) {
        console.log(data);
        console.log("ERROR");
        event.preventDefault();
    }

    function onSubmit(data: any, event: any) {
        console.log("registering user with data: " + data);
        register_user(data.name, data.email, data.password);
        event.preventDefault();
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
            <form onSubmit={handleSubmit(onSubmit, onError)}>
                <input
                    type="text"
                    placeholder="Name"
                    name="name"
                    ref={register({required: true})}
                    />
                {errors.name && <span className="errorMsg">Name required.</span>}
                <br />
                <input
                    type="text"
                    placeholder="Email"
                    name="email"
                    ref={register({required: true, maxLength: 80, pattern: {
                        value: /^\S+@\S+\.\S+$/i,
                        message: "Invalid email address."
                    }})} />
                {errors.email && <span className="errorMsg">{errors.email.message}</span>}
                <br />
                <input
                    type="text"
                    placeholder="Password"
                    name="password"
                    ref={register({required: true})} />
                {errors.password && <span className="errorMsg">Password required.</span>}
                <br />
                <input
                    type="text"
                    placeholder="Confirm Password"
                    name="confirmPassword"
                    ref={register({required: true})} />
                {errors.confirmPassword && <span className="errorMsg">Password required.</span>}
                <br />
                <input type="submit" value="Register" />
            </form>
        </ReactModal>
    )
}

export default Login;