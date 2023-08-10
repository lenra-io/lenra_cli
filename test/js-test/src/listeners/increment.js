'use strict'

import { Counter } from "../classes/Counter.js";

/**
 * 
 * @param {import("@lenra/app-server").props} props 
 * @param {import("@lenra/app-server").event} _event 
 * @param {import("@lenra/app-server").Api} api
 * @returns 
 */
export default async function(props, _event, api) {
    let counter = await api.data.getDoc(Counter, props.id);
    counter.count += 1;
    await api.data.updateDoc(counter);
    return {};
}