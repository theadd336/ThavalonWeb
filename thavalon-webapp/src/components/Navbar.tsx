import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import React from 'react';
import {NavLink} from 'react-router-dom';
import styled from 'styled-components';
import {faBars} from '@fortawesome/free-solid-svg-icons';

interface NavbarProps {
    loggedIn: boolean
    useMobileMenu: boolean
    setUseMobileMenu: React.Dispatch<React.SetStateAction<boolean>>
};

export function Navbar(props: NavbarProps) {
    // handle mobile menu functionality
    let topnavClasses: string = "topnav"
    if (props.useMobileMenu) {
        topnavClasses += " responsive";
    }
    return (
        <NavbarContainer id="navbar" className={topnavClasses}>
            <NavbarItemLeft exact to="/" activeClassName="active" id="homeLink">Home</NavbarItemLeft>
            <NavbarItemLeft to="/rules" activeClassName="active">Rules</NavbarItemLeft>
            {!props.loggedIn &&
                <NavbarItemRight to="/login" activeClassName="active">Log In</NavbarItemRight>
            }
            <NavbarItemRight to="" className="icon" onClick={() => props.setUseMobileMenu(!props.useMobileMenu)}>
                    <FontAwesomeIcon icon={faBars} />
            </NavbarItemRight>
            {props.loggedIn &&
                <RightSpan id="rightContainer">
                    <SubNavbarItemLeft to="/stats" className="nav-item" activeClassName="active">Stats</SubNavbarItemLeft>
                    <SubNavbarItemLeft to="/account" className="nav-item" activeClassName="active">Account</SubNavbarItemLeft>
                    <SubNavbarItemLeft to="/logout" className="nav-item" activeClassName="active">Log Out</SubNavbarItemLeft>
                </RightSpan>
            }
        </NavbarContainer>
    );
}

const RightSpan = styled.span`
    display: inline-block;
    float: right;

    @media screen and (max-width: 600px) {
        float: none;
        width: 100%;
    }
`;

const NavbarContainer = styled.div`
    background-color: #333;
    overflow: hidden;

    @media screen and (max-width: 600px) {
        &.responsive {
            a, #rightContainer a  {
                &.icon {
                    position: absolute;
                    right: 0;
                    top: 0;
                }

                float: none;
                display: block;
                text-align: left;
            }
        }
    }

    .icon {
        display: none;
    }
`;

const NavbarItem = styled(NavLink)`
    display: block;
    color: #F2F2F2;
    text-align: center;
    padding: 14px 16px;
    text-decoration: none;
    font-size: 17px;

    &:hover {
        background-color: #DDD;
        color: #000;
    }

    &:active {
        background-color: #4CAF50;
        color: #FFF;
    }

    &.active {
        background-color: #4CAF50;
        color: #FFF;
    }
`;

const NavbarItemLeft = styled(NavbarItem)`
    float: left;
    
    @media screen and (max-width: 600px) {
        &:not(:first-child) {
            display: none;
        }

        &.icon {
            float: right;
            display: block;
        }
    }
`;

// so can put in div floated to right, and not have first child appear in responsive mode
const SubNavbarItemLeft = styled(NavbarItem)`
    float: left;
    
    @media screen and (max-width: 600px) {
        display: none;

        &.icon {
            float: right;
            display: block;
        }
    }
`;


const NavbarItemRight = styled(NavbarItem)`
    float: right;

    @media screen and (max-width: 600px) {
        &:not(:first-child) {
            display: none;
        }

        &.icon {
            float: right;
            display: block;
        }
    }
`;
