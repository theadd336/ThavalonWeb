import React, { useEffect, useState } from 'react';
import { Navbar } from './components/Navbar';
import { Switch, Route } from 'react-router-dom';
import { Login } from './components/Login';
import { Logout } from './components/Logout';
import { Register } from './components/Register';
import { Home } from './components/Home';
import { AccountManager, HttpResponse } from './utils/AccountManager';
import { Account } from './components/Account';
import { CreateJoinGameModal } from './components/gameCreation';
import ReactModal from 'react-modal';

import "bootstrap/dist/css/bootstrap.min.css";
// Used by react modal for screen readers
ReactModal.setAppElement("#root");

function App() {
  const [loggedIn, setLoggedIn] = useState(false);
  const [useMobileMenu, setUseMobileMenu] = useState(false);
  const [showCreatePlayModal, setShowCreatePlayModal] = useState(false);
  // check logged in status within useEffect to not enter render loop
  useEffect(() => {
    const accountManager = AccountManager.getInstance();
    accountManager.checkLoggedIn().then((httpResponse: HttpResponse) => {
      // calling set logged in will, on success, trigger a timer to regularly check refresh token
      setLoggedIn(httpResponse.result);
    });
  })


  return (
    <div>
      <Navbar loggedIn={loggedIn} useMobileMenu={useMobileMenu} setUseMobileMenu={setUseMobileMenu} />
      <Switch>
        <Route path="/" exact>
          <Home />
        </Route>
        <Route path="/rules">
          <h1>Rules</h1>
        </Route>
        <Route path="/account">
          <Account />
        </Route>
        <Route path="/play">
          <CreateJoinGameModal show={showCreatePlayModal} />
        </Route>
        <Route path="/login" render={
          (_) => <Login setLoggedIn={setLoggedIn} />
        }>
        </Route>
        <Route path="/logout">
          <Logout setLoggedIn={() => setLoggedIn(false)} />
        </Route>
        <Route path="/register" render={
          (_) => <Register setLoggedIn={setLoggedIn} />
        }>
        </Route>
      </Switch>
    </div >
  );
}

export default App;
