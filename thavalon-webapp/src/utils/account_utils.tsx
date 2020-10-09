let account_logged_in = false;

export async function say_hello() {
    console.log("HI!");
    let response = await fetch("http://localhost:8001/api/hi");
    console.log(response);
    let body = await response.text();
    console.log(body);
};

export function is_logged_in() {
    console.log("logged in?");
    return account_logged_in;
};

export function log_in() {
    console.log("Logging in");
    account_logged_in = true;
    return true; // true indicates successfully logged in
};

export function log_out() {
    console.log("Logging out");
    account_logged_in = false;
    return true; // true indicates successfully logged out
};

export async function register_user(name: string, email: string, password: string) {
    console.log("Registering user! Name: " + name + ", email: " + email);
    let add_user_dict = {
        "email": email,
        "password": password,
        "displayName": name
    }

    let response = await fetch("/api/add/user", {
        method: "POST",
        body: JSON.stringify(add_user_dict),
        headers: {
            "Content-Type": "application/json"
        },
        credentials: "include"
    }).then((response) => {
        console.log(document.cookie);
        console.log(response);
        for (const header of response.headers.entries()) {
            console.log(header);
        }
        let auth_headers = response.headers.get("authentication");
        console.log(auth_headers);
    });
}