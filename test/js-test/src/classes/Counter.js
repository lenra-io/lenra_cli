import { Data } from "@lenra/app-server";

export class Counter extends Data {
    /**
     * 
     * @param {string} user 
     * @param {number} count 
     */
    constructor(user, count) {
        super();
        this.user = user;
        this.count = count;
    }
}