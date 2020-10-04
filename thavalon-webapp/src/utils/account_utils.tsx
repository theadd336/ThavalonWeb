let account_logged_in = false;

export function is_logged_in() {
    console.log("logged in?");
    return account_logged_in;
}

export function log_in() {
    console.log("Logging in");
    account_logged_in = true;
    return true; // true indicates successfully logged in
}

export function log_out() {
    console.log("Logging out");
    account_logged_in = false;
    return true; // true indicates successfully logged out
}