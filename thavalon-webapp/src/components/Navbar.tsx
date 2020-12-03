import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import React, {MouseEvent} from 'react';
import {NavLink} from 'react-router-dom';
import {faBars} from '@fortawesome/free-solid-svg-icons';
import "../styles/Navbar.scss";

interface NavbarProps {
    loggedIn: boolean
    useMobileMenu: boolean
    setUseMobileMenu: React.Dispatch<React.SetStateAction<boolean>>
};

export function Navbar(props: NavbarProps): JSX.Element {
    // handle mobile menu functionality
    let topnavClasses: string = "topnav"
    if (props.useMobileMenu) {
        topnavClasses += " responsive";
    }

    /**
     * When opening menu, toggle if mobile menu should be displayed or not.
     * Also suppress click event, in order to avoid changing current page.
     * 
     * @param event The event generated by opening the menu.
     */
    function openMenu(event: MouseEvent<HTMLAnchorElement>) {
        props.setUseMobileMenu(!props.useMobileMenu);
        event.preventDefault();
    }

    return (
        <div id="navbarContainer" className={topnavClasses}>
            <NavLink exact to="/" className="navbarItemLeft" activeClassName="active" id="homeLink">Home</NavLink>
            <NavLink to="/rules" className="navbarItemLeft" activeClassName="active">Rules</NavLink>
            {!props.loggedIn &&
                <NavLink to="/login" className="navbarItemRight" activeClassName="active">Log In</NavLink>
            }
            <NavLink to="" className="icon navbarItemRight" onClick={openMenu}>
                    <FontAwesomeIcon icon={faBars} />
            </NavLink>
            {props.loggedIn &&
                <span id="rightContainer">
                    <NavLink to="/profile" className="navbarItemLeft subNavbarItemLeft" activeClassName="active">Profile</NavLink>
                    <NavLink to="/account" className="navbarItemLeft subNavbarItemLeft" activeClassName="active">Account</NavLink>
                    <NavLink to="/logout" className="navbarItemLeft subNavbarItemLeft" activeClassName="active">Log Out</NavLink>
                </span>
            }
        </div>
    );
}