import React, { useState } from 'react';
import Navbar from './components/Navbar';
import { Switch, Route } from 'react-router-dom';
import Login from './components/Login';
import Logout from './components/Logout';
import Register from './components/Register';

function App() {
  // TODO: Replace with api call to 
  const [loggedIn, setLoggedIn] = useState(false);

  return (
    <div>
      <Navbar loggedIn={loggedIn} />
      <Switch>
        <Route path="/" exact>
          <h1>Home</h1>
        </Route>
        <Route path="/rules">
          <h1>Rules</h1>
        </Route>
        <Route path="/login" render={
          (routeProps) => <Login setLoggedIn={setLoggedIn} />
        }>
        </Route>
        <Route path="/logout">
          <Logout setLoggedIn={() => setLoggedIn(false)} />
        </Route>
        <Route path="/register" render={
          (routeProps) => <Register setLoggedIn={setLoggedIn} />
        }>
        </Route>
      </Switch>
    </div>
  );
}

export default App;
