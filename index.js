"use strict";

const { promisify } = require("util");
const { wtNew, wtGetReady, wtGetStats, wtClose } = require("./index.node");

class WebTransport {
    constructor() {
        this.wt = wtNew();
    }

    get ready() {
        return wtGetReady.call(this.wt);
    }

    getStats() {
        return wtGetStats.call(this.wt);
    }

    close() {
        return wtClose.call(this.wt);
    }
}

module.exports = WebTransport;
