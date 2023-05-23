"use strict";

const { promisify } = require("util");
const { wtNew, wtGetStats, wtClose } = require("./index.node");

class WebTransport {
    constructor() {
        this.wt = wtNew();
        console.log(this.wt);
    }

    getStats() {
        console.log(this.wt);
        return wtGetStats(this.wt);
    }

    close() {
        console.log(this.wt);
        return wtClose(this.wt);
    }
}

module.exports = WebTransport;
