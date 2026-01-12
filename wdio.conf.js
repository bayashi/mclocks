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
            args: [
                '--disable-web-security',
                '--disable-features=IsolateOrigins,site-per-process',
                '--unsafely-treat-insecure-origin-as-secure=http://localhost:1420'
            ],
            prefs: {
                'profile.default_content_setting_values.clipboard': 0,
                'profile.content_settings.exceptions.clipboard': {
                    'http://localhost:1420,*': {
                        'last_modified': '13317004800000000',
                        'setting': 1
                    },
                    'http://localhost,*': {
                        'last_modified': '13317004800000000',
                        'setting': 1
                    },
                    'http://127.0.0.1:1420,*': {
                        'last_modified': '13317004800000000',
                        'setting': 1
                    }
                }
            }
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
    afterTest: async function (test, context, { error, result, duration, passed, retries }) {
        // If test failed, keep browser open for debugging
        if (!passed) {
            console.log('\n=== Test failed - Browser will stay open for 30 seconds for debugging ===')
            console.log('Test:', test.title)
            if (error) {
                console.log('Error:', error.message)
            }
            // Keep browser open for 30 seconds to allow copying error messages
            await browser.pause(30000)
        }
    },

    onComplete: function(exitCode, config, capabilities, results) {
    }
}
