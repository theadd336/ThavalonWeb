const STATUS_OK: number = 200;
const STATUS_CREATED: number = 201;
const STATUS_RESET_CONTENT: number = 205;
const STATUS_UNAUTHORIZED: number = 401;
const STATUS_NOT_ACCEPTABLE: number = 406;
const STATUS_CONFLICT: number = 409;
const STATUS_INTERNAL_SERVER_ERROR: number = 500;

interface AddUserInfo {
    "displayName": string,
    "email": string,
    "password": string,
};

interface LogInInfo {
    "email": string,
    "password": string,
};

interface JwtType {
    "token_type": string,
    "access_token": string,
    "expires_at": number,
};

export interface HttpResponse {
    "result": boolean, // true if successful http query, false otherwise
    "message": string, // message will contain error message if result is false, otherwise blank
};

export class AccountManager {
    private static instance: AccountManager;
    private token: string;
    private expiresAt: number;

    /**
     * Get the account manager instance, creating one if needed.
     * 
     * @returns The instance of the AccountManager.
     */
    public static getInstance(): AccountManager {
        if (!AccountManager.instance) {
            AccountManager.instance = new AccountManager();
        }

        return AccountManager.instance;
    }

    private constructor() {
        this.token = "";
        this.expiresAt = 0;
    }

    /**
     * Set the jwt info. Will also set a timer to get the next JWT after 60 seconds before this one expires.
     * @param jwt The given JWT.
     * @param callback If passed in, will call the callback 60 seconds before jwt is set to expire
     */
    private setJwtInfo(jwt: JwtType, callback?: () => Promise<HttpResponse>): void {
        this.token = jwt.access_token;
        this.expiresAt = jwt.expires_at;

        const currUnixTime = Math.floor(Date.now() / 1000);
        // Take the diff between expires at and now, and subtract 60 seconds
        // This is the timeout for when we should recheck the refresh token
        const refreshTimeout = (this.expiresAt - currUnixTime) - 890;
        if (callback !== undefined) {
            setTimeout(callback, refreshTimeout * 1000);
        }
    }
    /**
     * Queries the server to see if refresh token we currently have is valid.
     */
    private async checkRefreshToken(): Promise<HttpResponse> {
        console.log("Checking refresh token!");
        const httpResponse: HttpResponse = {
            "result": true,
            "message": "",
        }
        // returns 500 or 401 or 200
        return await fetch("/api/auth/refresh", {
            method: "POST",
            credentials: "include"
        }).then((response) => {
            if (response.status === STATUS_OK) {
                response.json().then((jwt: JwtType) => {
                    // use anonymous function to get around this being unbound
                    this.setJwtInfo(jwt, () => (this.checkRefreshToken()));
                });
            } else if (response.status === STATUS_UNAUTHORIZED) {
                httpResponse.result = false;
                httpResponse.message = "Unauthorized when trying to refresh token";
            } else if (response.status === STATUS_INTERNAL_SERVER_ERROR) {
                httpResponse.result = false;
                httpResponse.message = "Internal server error when refreshing token";
            } else {
                httpResponse.result = false;
                httpResponse.message = "Unexpected return code from server: " + response.status;
            }
            return httpResponse;
        }).catch((error) => {
            console.log("Failed to refresh token, error is: " + error);
            httpResponse.result = false;
            httpResponse.message = "Unable to refresh token, try again";
            return httpResponse;
        });
    }

    public async checkLoggedIn(): Promise<HttpResponse> {
        return await this.checkRefreshToken();
    }

    /**
     * Registers a user with the given info.
     * 
     * @param name The name of the user
     * @param email The email address of the user
     * @param password The password of the user
     * @returns HttpResponse with result (true on success, false otherwise) and message set if result is false
     */
    public async registerUser(name: string, email: string, password: string): Promise<HttpResponse> {
        // parameters for registering user
        const addUserInfo: AddUserInfo = {
            "displayName": name,
            "email": email,
            "password": password
        }

        const httpResponse: HttpResponse = {
            "result": true,
            "message": ""
        }
        // Following end point can return 201 on successful add or 406 on reject or 500 if everything's broken or 409 if duplicate account
        return await fetch("/api/add/user", {
            method: "POST",
            body: JSON.stringify(addUserInfo),
            headers: {
                "Content-Type": "application/json"
            }
        }).then((response) => {
            // On success, set jwt info. On fail, set error messages to return to user.
            if (response.status === STATUS_CREATED) {
                response.json().then((jwt: JwtType) => {
                    this.setJwtInfo(jwt, this.checkRefreshToken);
                });
            } else if (response.status === STATUS_NOT_ACCEPTABLE) {
                httpResponse.result = false;
                httpResponse.message = "Invalid email or password";
            } else if (response.status === STATUS_CONFLICT) {
                httpResponse.result = false;
                httpResponse.message = "Invalid email - already in use";
            } else if (response.status === STATUS_INTERNAL_SERVER_ERROR) {
                httpResponse.result = false;
                httpResponse.message = "Unable to create account, try again";
            } else {
                httpResponse.result = false;
                httpResponse.message = "Unexpected return code from server: " + response.status;
            }
            return httpResponse;
        }).catch((error) => {
            console.log("Failed to register user, error is: " + error);
            httpResponse.result = false;
            httpResponse.message = "Unable to register, try again";
            return httpResponse;
        });
    }

    /**
     * Logins the user with the given info.
     * @param email The email of the user attempting to log in.
     * @param password The password of the user attempting to log in.
     * @returns HttpResponse with result (true on success, false otherwise) and message set if result is false
     */
    public async loginUser(email: string, password: string): Promise<HttpResponse> {
        // parameters for logging in user
        const logInInfo: LogInInfo = {
            "email": email,
            "password": password
        }

        const httpResponse: HttpResponse = {
            "result": true,
            "message": ""
        }

        return await fetch("/api/auth/login", {
            method: "POST",
            body: JSON.stringify(logInInfo),
            headers: {
                "Content-Type": "application/json"
            }
        }).then((response) => {
            if (response.status === STATUS_OK) {
                response.json().then((jwt: JwtType) => {
                    this.setJwtInfo(jwt);
                });
            } else if (response.status === STATUS_UNAUTHORIZED) {
                httpResponse.result = false;
                httpResponse.message = "Invalid email or password";
            } else if (response.status === STATUS_INTERNAL_SERVER_ERROR) {
                httpResponse.result = false;
                httpResponse.message = "Unable to log in, try again";
            } else {
                httpResponse.result = false;
                httpResponse.message = "Unexpected return code from server: " + response.status;
            }
            return httpResponse;
        }).catch((error) => {
            console.log("Failed to login user, error is: " + error);
            httpResponse.result = false;
            httpResponse.message = "Unable to login, try again";
            return httpResponse;
        });
    }

    /**
     * Log out the currently logged in user.
     * 
     * @returns HttpResponse with result (true on success, false otherwise) and message set if result is false
     */
    public async logoutUser(): Promise<HttpResponse> {
        const httpResponse: HttpResponse = {
            "result": true,
            "message": ""
        }

        return await fetch("/api/auth/logout", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            credentials: "include"
        }).then((response) => {
            if (response.status === STATUS_RESET_CONTENT) {
                this.token = "";
                this.expiresAt = 0;
            } else {
                httpResponse.result = false;
                httpResponse.message = "Unexpected return code from server: " + response.status;
            }
            return httpResponse;
        }).catch((error) => {
            console.log("Failed to logout user, error is: " + error);
            httpResponse.result = false;
            httpResponse.message = "Unable to logout, try again";
            return httpResponse;
        });
    }
}
