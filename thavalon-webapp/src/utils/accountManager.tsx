const STATUS_CREATED: number = 201;
const STATUS_NOT_ACCEPTABLE: number = 406;
const STATUS_CONFLICT: number = 409;
const STATUS_INTERNAL_SERVER_ERROR: number = 500;

export interface JwtType {
    "token_type": string,
    "access_token": string,
    "epxires_at": number
};

export interface RegisterResponse {
    "result": boolean,
    "message": string, // message will contain error message if result is false, otherwise blank
}

class AccountManager {
    private static instance: AccountManager;
    private token: string;
    private expiresAt: number;

    private constructor() {
        this.token = "";
        this.expiresAt = 0;
    }

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

    /**
     * Registers a user with the given info.
     * 
     * @param name The name of the user
     * @param email The email address of the user
     * @param password The password of the user
     * @returns True on successful register, false otherwise.
     */
    public async registerUser(name: string, email: string, password: string): Promise<RegisterResponse> {
        // parameters for registering user
        let addUserMap = {
            "displayName": name,
            "email": email,
            "password": password
        }

        let registerResponse: RegisterResponse = {
            "result": true,
            "message": ""
        }
        // Following end point can return 201 on successful add or 406 on reject or 500 if everything's broken or 409 if duplicate account
        return await fetch("/api/add/user", {
            method: "POST",
            body: JSON.stringify(addUserMap),
            headers: {
                "Content-Type": "application/json"
            },
            credentials: "include"
        }).then((response) => {
            if (response.status === STATUS_CREATED) {
                response.json().then((jwt: JwtType) => {
                    this.token = jwt.access_token;
                    this.expiresAt = jwt.epxires_at;
                });
            } else if (response.status === STATUS_NOT_ACCEPTABLE) {
                registerResponse.result = false;
                registerResponse.message = "Invalid email or password";
            } else if (response.status === STATUS_CONFLICT) {
                registerResponse.result = false;
                registerResponse.message = "Invalid email - already in use";
            } else if (response.status === STATUS_INTERNAL_SERVER_ERROR) {
                registerResponse.result = false;
                registerResponse.message = "Unable to create account, try again";
            }
            return registerResponse;
        }).catch((error) => {
            console.log("Failed to register user, error is: " + error);
            registerResponse.result = false;
            registerResponse.message = "Unable to register, try again";
            return registerResponse;
        });
    }
}

export default AccountManager;