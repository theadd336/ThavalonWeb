import React, { useEffect, useState } from 'react';
import { Navbar } from './components/Navbar';
import { Switch, Route, Redirect } from 'react-router-dom';
import { Login } from './components/Login';
import { Logout } from './components/Logout';
import { Register } from './components/Register';
import { Home } from './components/Home';
import { AccountManager, HttpResponse } from './utils/AccountManager';
import { CreateJoinGameModal } from './components/gameCreation';
import { GameContainer } from "./components/gameContainer";
import ReactModal from 'react-modal';

import "bootstrap/dist/css/bootstrap.min.css";
import { Profile } from './components/profileComponents/Profile';
// Used by react modal for screen readers
ReactModal.setAppElement("#root");

function App() {
  // state for checking if logged in
  const [loggedIn, setLoggedIn] = useState(false);
  // state for checking if logged in already checked via useEffect, to prevent it from briefly loading
  // the register page before redirect to account page if user is logged in
  const [checkedLoggedIn, setCheckedLoggedIn] = useState(false);
  // state for checking if should display mobile navbar menu
  const [useMobileMenu, setUseMobileMenu] = useState(false);
  // state for determining if the login modal shown, controlled by navbar click
  const [showLoginModal, setShowLoginModal] = useState(false);
  // state for determining if the create/join game modal is shown, controlled by navbar click.
  const [showCreateJoinGameModal, setShowCreateJoinGameModal] = useState(false);

  // check logged in status within useEffect to not enter render loop
  useEffect(() => {
    const accountManager = AccountManager.getInstance();
    accountManager.checkLoggedIn().then((httpResponse: HttpResponse) => {
      // calling set logged in will, on success, trigger a timer to regularly check refresh token
      setLoggedIn(httpResponse.result);
      setCheckedLoggedIn(true);
    });
  })

  function registerPage(): JSX.Element {
    if (checkedLoggedIn) {
      if (loggedIn) {
        return <Redirect to="/account" />;
      } else {
        return <Register setLoggedIn={setLoggedIn} setShowLoginModal={setShowLoginModal} />;
      }
    }
    return <></>;
  }

  return (
    <div>
      <Navbar
        loggedIn={loggedIn}
        useMobileMenu={useMobileMenu}
        setUseMobileMenu={setUseMobileMenu}
        setShowLoginModal={setShowLoginModal}
        setShowCreateJoinGameModal={setShowCreateJoinGameModal}
      />
      <Switch>
        <Route path="/" exact>
          <Home />
        </Route>
        <Route path="/rules">
          <h1>Rules</h1>
        </Route>
        <Route path="/profile">
          <Profile />
        </Route>
        <Route path="/logout">
          <Logout setLoggedIn={() => setLoggedIn(false)} />
        </Route>
        <Route path="/register" render={() => registerPage()} />
        <Route path="/game" component={GameContainer}>
        </Route>
      </Switch>
      {showLoginModal &&
        <Login
          setLoggedIn={setLoggedIn}
          setShowLoginModal={setShowLoginModal}
          showLoginModal={showLoginModal} />}
      {showCreateJoinGameModal &&
        <CreateJoinGameModal
          isLoggedIn={loggedIn}
          setShowLoginModal={setShowLoginModal}
          show={showCreateJoinGameModal}
          setOpen={setShowCreateJoinGameModal} />}
    </div>
  );
}

export default App;
