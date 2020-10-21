import React, { useEffect, useState } from 'react';
import Navbar from './components/Navbar';
import { Switch, Route } from 'react-router-dom';
import Login from './components/Login';
import Logout from './components/Logout';
import Register from './components/Register';
import { say_hello } from './utils/account_utils';

function App() {
  const [loggedIn, setLoggedIn] = useState(false);
  useEffect(
    () => setLoggedIn(loggedIn),
    [loggedIn]
  )
  return (
    <div>
      <Navbar loggedIn={loggedIn} />
      <Switch>
        <Route path="/" exact>
          <h1>Home</h1>
        </Route>
        <Route path="/rules">
          <h1>Rules</h1>
          {say_hello()}
        </Route>
        <Route path="/login">
          <Login setLoggedIn={() => setLoggedIn(true)} />
        </Route>
        <Route path="/logout">
          <Logout setLoggedIn={() => setLoggedIn(false)} />
        </Route>
        <Route path="/register">
          <Register />
        </Route>
      </Switch>
    </div>
  );
}

export default App;
