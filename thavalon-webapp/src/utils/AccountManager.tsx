import { GameSocket } from "./GameSocket";

enum STATUS {
    OK = 200,
    CREATED = 201,
    RESET_CONTENT = 205,
    UNAUTHORIZED = 401,
    NOT_ACCEPTABLE = 406,
    CONFLICT = 409,
    INTERNAL_SERVER_ERROR = 500
};

interface AddUserInfo {
    displayName: string,
    email: string,
    password: string,
};

interface LogInInfo {
    email: string,
    password: string,
};

interface JwtType {
    token_type: string,
    access_token: string,
    expires_at: number,
};

interface CreateGameResponse {
    friendCode: string,
}

interface JoinGameInfo {
    friendCode: string,
    displayName: string,
}

interface JoinGameResponse {
    socketUrl: string,
}

export interface HttpResponse {
    result: boolean, // true if successful http query, false otherwise
    message: string, // message will contain error message if result is false, otherwise blank
};

/**
 * A singleton containing code related to the user account, such as
 * log in and joining a game.
 */
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
    private setJwtInfo(jwt: JwtType): void {
        this.token = jwt.access_token;
        this.expiresAt = jwt.expires_at;
    }

    /**
     * Logic for checking refresh token.
     * @returns A promise with an HttpResponse, containing server code.
     */
    private async checkRefreshToken(): Promise<HttpResponse> {
        const httpResponse: HttpResponse = {
            result: true,
            message: "",
        }
        // returns 500 or 401 or 200
        const response: Response = await fetch("/api/auth/refresh", {
            method: "POST",
            credentials: "include"
        });

        switch (response.status) {
            case STATUS.OK: {
                // this.setJwtInfo.bind(this);
                const jwt: JwtType = await response.json();
                this.setJwtInfo(jwt);
                break;
            }
            case STATUS.UNAUTHORIZED: {
                httpResponse.result = false;
                httpResponse.message = "Unauthorized when trying to refresh token";
                break;
            }
            case STATUS.INTERNAL_SERVER_ERROR: {
                httpResponse.result = false;
                httpResponse.message = "Internal server error when refreshing token";
                break;
            }
            default: {
                console.log("Unexpected return code from server: " + response.status);
                httpResponse.result = false;
                httpResponse.message = "Request failed, try again.";
                break;
            }
        }
        // log any non-OK statuses, and clear jwt info
        if (!httpResponse.result) {
            console.log("Invalid refresh token, Reason: " + httpResponse.message);
            this.token = "";
            this.expiresAt = 0;
        }
        return httpResponse;
    }

    /**
     * Queries the server to see if refresh token we currently have is valid. Will do this
     * on a timer so long as JWT is valid.
     **/
    private checkRefreshTokenOnTimer(): void {
        const currUnixTime = Math.floor(Date.now() / 1000);
        const refreshTimeout = (this.expiresAt - currUnixTime) - 60;
        // in timeout, use anonymous function so checkRefreshToken has access to this
        // when timer ends, will check refresh token and call this function again if
        // refresh token was valid
        setTimeout(() => {
            this.checkRefreshToken().then((httpResponse: HttpResponse) => {
                if (httpResponse.result) {
                    this.checkRefreshTokenOnTimer();
                }
            });
        }, refreshTimeout * 1000);
    }

    /**
     * Checks whether user is logged in via refresh token. If so, will set a timer to check
     * refresh token regularly.
     * @param setTimer if True, will set a timer to recheck the token regularly if the user is logged in
     * @returns A promise with an HttpResponse, indicating if user is logged in and any errors.
     */
    public async checkLoggedIn(setTimer = true): Promise<HttpResponse> {
        let httpResponse = await this.checkRefreshToken();
        if (setTimer && httpResponse.result) {
            this.checkRefreshTokenOnTimer();
        }
        return httpResponse;
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
            displayName: name,
            email: email,
            password: password
        }

        const httpResponse: HttpResponse = {
            result: true,
            message: ""
        }
        // Following end point can return 201 on successful add or 406 on reject or 500 if everything's broken or 409 if duplicate account
        const response: Response = await fetch("/api/add/user", {
            method: "POST",
            body: JSON.stringify(addUserInfo),
            headers: {
                "Content-Type": "application/json"
            }
        })

        // On success, set jwt info. On fail, set error messages to return to user.
        switch (response.status) {
            case STATUS.CREATED: {
                const jwt: JwtType = await response.json();
                this.setJwtInfo(jwt);
                break;
            }
            case STATUS.NOT_ACCEPTABLE: {
                httpResponse.result = false;
                httpResponse.message = "Invalid email or password";
                return httpResponse;
            }
            case STATUS.CONFLICT: {
                httpResponse.result = false;
                httpResponse.message = "Invalid email - already in use";
                return httpResponse;
            }
            case STATUS.INTERNAL_SERVER_ERROR: {
                httpResponse.result = false;
                httpResponse.message = "Unable to create account, try again";
                return httpResponse;
            }
            default: {
                console.log("Unexpected return code from server: " + response.status);
                httpResponse.result = false;
                httpResponse.message = "Request failed, try again.";
                return httpResponse;
            }
        }
        httpResponse.result = false;
        httpResponse.message = "Unable to register, try again";
        return httpResponse;
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
            email: email,
            password: password
        }

        const httpResponse: HttpResponse = {
            result: true,
            message: ""
        }

        let response: Response = await fetch("/api/auth/login", {
            method: "POST",
            body: JSON.stringify(logInInfo),
            headers: {
                "Content-Type": "application/json"
            }
        });

        switch (response.status) {
            case STATUS.OK: {
                const jwt: JwtType = await response.json();
                this.setJwtInfo(jwt);
                break;
            }
            case STATUS.UNAUTHORIZED: {
                httpResponse.result = false;
                httpResponse.message = "Invalid email or password";
                return httpResponse;
            }
            case STATUS.INTERNAL_SERVER_ERROR: {
                httpResponse.result = false;
                httpResponse.message = "Unable to log in, try again";
                return httpResponse;
            }
            default: {
                console.log("Unexpected return code from server: " + response.status);
                httpResponse.result = false;
                httpResponse.message = "Request failed, try again.";
                return httpResponse;
            }
        }

        httpResponse.result = false;
        httpResponse.message = "Unable to login, try again";
        return httpResponse;
    }

    /**
     * Log out the currently logged in user.
     * 
     * @returns HttpResponse with result (true on success, false otherwise) and message set if result is false
     */
    public async logoutUser(): Promise<HttpResponse> {
        const httpResponse: HttpResponse = {
            result: true,
            message: ""
        }

        const response: Response = await fetch("/api/auth/logout", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            credentials: "include"
        });

        if (response.status === STATUS.RESET_CONTENT) {
            this.token = "";
            this.expiresAt = 0;
        } else {
            console.log("Unexpected return code from server: " + response.status);
            httpResponse.result = false;
            httpResponse.message = "Request failed, try again.";
        }
        return httpResponse;
    }

    /**
     * Creates a game made by the current user.
     */
    public async createGame(): Promise<HttpResponse> {
        const httpResponse: HttpResponse = {
            result: true,
            message: ""
        }

        const response: Response = await fetch("/api/add/game", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                "Authorization": "Basic " + this.token,
            }
        });

        if (response.status === STATUS.OK) {
            const createGameResponse: CreateGameResponse = await response.json();
            httpResponse.message = createGameResponse.friendCode;
        } else {
            console.log("Unexpected return code from server: " + response.status);
            httpResponse.result = false;
            httpResponse.message = "Request failed, try again.";
        }
        return httpResponse;
    }

    /**
     * Joins an existing game.
     *
     * @param friendCode the friend code for the game
     * @param displayName the display name of the user joining the game
     */
    public async joinGame(friendCode: string, displayName: string): Promise<HttpResponse> {
        const httpResponse: HttpResponse = {
            result: true,
            message: ""
        }

        const joinGameInfo: JoinGameInfo = {
            friendCode: friendCode,
            displayName: displayName,
        }

        const response: Response = await fetch("/api/join/game", {
            method: "POST",
            body: JSON.stringify(joinGameInfo),
            headers: {
                "Content-Type": "application/json",
                "Authorization": "Basic " + this.token,
            }
        });
        if (response.status === STATUS.OK) {
            const joinGameResponse: JoinGameResponse = await response.json();
            httpResponse.message = joinGameResponse.socketUrl;
        } else {
            console.log("Unexpected return code from server: " + response.status);
            httpResponse.result = false;
            httpResponse.message = "Request failed, try again.";
        }
        return httpResponse;
    }
}
