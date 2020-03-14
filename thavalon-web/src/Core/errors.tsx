export class InvalidPlayerError extends Error {
    constructor(errorMessage?: string) {
        super(errorMessage);
        Object.setPrototypeOf(this, new.target.prototype);
    }
}

export class MissingPropertyError extends Error {
    constructor(errorMessage?: string) {
        super(errorMessage);
        Object.setPrototypeOf(this, new.target.prototype);
    }
}

export class InvalidMissionError extends Error {
    constructor(errorMessage?: string) {
        super(errorMessage);
        Object.setPrototypeOf(this, new.target.prototype);
    }
}

export class ConnectionError extends Error {
    constructor(errorMessage?: string) {
        if (typeof errorMessage === undefined) {
            super("The connection is in a broken state.");
        } else {
            super(errorMessage);
        }
        Object.setPrototypeOf(this, new.target.prototype);
    }
}