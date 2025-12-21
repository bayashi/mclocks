export const config = {
    //
    // ====================
    // Runner Configuration
    // ====================
    //
    runner: 'local',
    
    //
    // ==================
    // Specify Test Files
    // ==================
    //
    specs: [
        './test/specs/**/*.js'
    ],
    
    //
    // ============
    // Capabilities
    // ============
    //
    capabilities: [{
        browserName: 'chrome',
        'goog:chromeOptions': {
            args: ['--disable-web-security', '--disable-features=IsolateOrigins,site-per-process']
        }
    }],
    
    //
    // ===================
    // Test Configurations
    // ===================
    //
    baseUrl: 'http://localhost:1420',
    
    logLevel: 'info',
    
    framework: 'mocha',
    
    reporters: ['spec'],
    
    mochaOpts: {
        ui: 'bdd',
        timeout: 60000
    },
    
    //
    // =====
    // Hooks
    // =====
    //
    onPrepare: async function (config, capabilities) {
        // Check if the application is running
        // Note: You need to start the app with `pnpm tauri dev` before running tests
        console.log('Make sure mclocks is running on http://localhost:1420')
        console.log('You can start it with: pnpm tauri dev')
    },
    
    onComplete: function(exitCode, config, capabilities, results) {
    }
}

