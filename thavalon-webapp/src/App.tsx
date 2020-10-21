import React, { useEffect, useState } from 'react';
import { Navbar } from './components/Navbar';
import { Switch, Route } from 'react-router-dom';
import { Login } from './components/Login';
import { Logout } from './components/Logout';
import { Register } from './components/Register';
import { Home } from './components/Home';
import { AccountManager, HttpResponse } from './utils/accountManager';

function App() {
  const [loggedIn, setLoggedIn] = useState(false);
  // check logged in status within useEffect to not enter render loop
  useEffect(() => {
    const accountManager = AccountManager.getInstance();
    accountManager.checkLoggedIn().then((httpResponse: HttpResponse) => {
      console.log("Result is: " + httpResponse.result);
      setLoggedIn(httpResponse.result);
    });  
  })


  return (
    <div>
      <Navbar loggedIn={loggedIn} />
      <Switch>
        <Route path="/" exact>
          <Home />
        </Route>
        <Route path="/rules">
          <h1>Rules</h1>
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
    </div>
  );
}

export default App;
