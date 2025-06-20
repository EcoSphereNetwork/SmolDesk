// signaling-server/monitoring.js
const os = require('os');
const process = require('process');

function getSystemStats() {
    return {
        timestamp: new Date().toISOString(),
        memory: {
            used: process.memoryUsage(),
            system: {
                total: os.totalmem(),
                free: os.freemem()
            }
        },
        cpu: {
            usage: process.cpuUsage(),
            load: os.loadavg()
        },
        uptime: process.uptime()
    };
}

module.exports = { getSystemStats };
