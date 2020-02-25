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