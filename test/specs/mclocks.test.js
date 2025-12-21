describe('mclocks Application Launch Test', () => {
    it('should launch the application and wait for it to be ready', async () => {
        // Connect to the application URL
        console.log('Connecting to http://localhost:1420...')
        await browser.url('/')
        
        // Wait for the page to load
        console.log('Waiting for page to load...')
        await browser.waitUntil(
            async () => {
                const readyState = await browser.execute(() => document.readyState)
                console.log(`Document ready state: ${readyState}`)
                return readyState === 'complete'
            },
            {
                timeout: 30000,
                timeoutMsg: 'Page did not load in time',
                interval: 1000
            }
        )
        
        // Verify that the main element exists
        console.log('Waiting for #mclocks element...')
        const mainElement = await $('#mclocks')
        await mainElement.waitForExist({ timeout: 30000 })
        
        // Debug: Check current DOM state
        const debugInfo = await browser.execute(() => {
            const mainEl = document.querySelector('#mclocks')
            return {
                exists: mainEl !== null,
                innerHTML: mainEl ? mainEl.innerHTML.substring(0, 200) : 'null',
                hasUl: mainEl ? mainEl.querySelector('ul') !== null : false,
                childrenCount: mainEl ? mainEl.children.length : 0
            }
        })
        console.log('Debug info:', JSON.stringify(debugInfo, null, 2))
        
        // Wait for the application to initialize and add ul element
        // Also verify that no error message is displayed
        console.log('Waiting for application initialization...')
        await browser.waitUntil(
            async () => {
                const result = await browser.execute(() => {
                    const mainEl = document.querySelector('#mclocks')
                    if (!mainEl) {
                        return { initialized: false, reason: 'mainEl is null' }
                    }
                    
                    // Check if an error message is displayed
                    const textContent = mainEl.textContent || ''
                    if (textContent.startsWith('Err:')) {
                        return { initialized: false, reason: 'Error detected', error: textContent }
                    }
                    
                    const ul = mainEl.querySelector('ul')
                    if (!ul) {
                        return { 
                            initialized: false, 
                            reason: 'ul not found', 
                            innerHTML: mainEl.innerHTML.substring(0, 200),
                            textContent: textContent.substring(0, 100)
                        }
                    }
                    
                    // Check if clock elements exist within ul element
                    const clockElements = ul.querySelectorAll('[id^="mclk-"]')
                    return { 
                        initialized: true, 
                        ulExists: true, 
                        clockCount: clockElements.length 
                    }
                })
                
                if (!result.initialized) {
                    console.log(`Not initialized yet: ${result.reason || 'unknown'}`)
                    if (result.innerHTML) {
                        console.log(`Current innerHTML: ${result.innerHTML}`)
                    }
                    if (result.textContent) {
                        console.log(`Current textContent: ${result.textContent}`)
                    }
                    if (result.error) {
                        console.error(`Application error: ${result.error}`)
                    }
                } else {
                    console.log(`Application initialized successfully with ${result.clockCount} clock(s)`)
                }
                
                return result.initialized === true
            },
            {
                timeout: 60000,
                timeoutMsg: 'Application did not initialize in time',
                interval: 1000
            }
        )
        
        // Verify that clock elements exist (e.g., mclk-0)
        console.log('Waiting for clock elements...')
        const clockElement = await $('[id^="mclk-"]')
        await clockElement.waitForExist({ timeout: 10000 })
        
        console.log('mclocks application is ready and initialized')
    })
    
    it('should render clocks and verify they are updating', async () => {
        // Connect to the application URL
        console.log('Connecting to http://localhost:1420...')
        await browser.url('/')
        
        // Wait for the application to initialize
        const mainElement = await $('#mclocks')
        await mainElement.waitForExist({ timeout: 30000 })
        
        // Wait for clock elements to be rendered
        await browser.waitUntil(
            async () => {
                const clockElements = await browser.execute(() => {
                    return Array.from(document.querySelectorAll('[id^="mclk-"]'))
                        .map(el => ({
                            id: el.id,
                            textContent: el.textContent.trim(),
                            hasContent: el.textContent.trim().length > 0
                        }))
                })
                return clockElements.length > 0 && clockElements.every(clock => clock.hasContent)
            },
            {
                timeout: 30000,
                timeoutMsg: 'Clock elements were not rendered',
                interval: 1000
            }
        )
        
        // Get initial clock values
        const initialClocks = await browser.execute(() => {
            return Array.from(document.querySelectorAll('[id^="mclk-"]'))
                .map(el => ({
                    id: el.id,
                    textContent: el.textContent.trim()
                }))
        })
        
        console.log('Initial clock values:', JSON.stringify(initialClocks, null, 2))
        
        // Verify that at least one clock is rendered
        expect(initialClocks.length).toBeGreaterThan(0, 'At least one clock should be rendered')
        
        // Verify that clocks have content
        initialClocks.forEach(clock => {
            expect(clock.textContent.length).toBeGreaterThan(0, `Clock ${clock.id} should have content`)
        })
        
        // Wait for clocks to update (at least 2 seconds to ensure update)
        console.log('Waiting for clocks to update...')
        await browser.pause(2500)
        
        // Get updated clock values
        const updatedClocks = await browser.execute(() => {
            return Array.from(document.querySelectorAll('[id^="mclk-"]'))
                .map(el => ({
                    id: el.id,
                    textContent: el.textContent.trim()
                }))
        })
        
        console.log('Updated clock values:', JSON.stringify(updatedClocks, null, 2))
        
        // Verify that clocks have been updated
        // For time clocks, the time should have changed
        // For countdown clocks, the countdown should have decreased
        let clocksUpdated = false
        
        for (let i = 0; i < initialClocks.length; i++) {
            const initial = initialClocks[i]
            const updated = updatedClocks.find(c => c.id === initial.id)
            
            if (updated && updated.textContent !== initial.textContent) {
                clocksUpdated = true
                console.log(`Clock ${initial.id} updated: "${initial.textContent}" -> "${updated.textContent}"`)
                break
            }
        }
        
        // At least one clock should have updated
        expect(clocksUpdated).toBe(true, 'At least one clock should have updated after waiting')
        
        // Verify that all clocks still have content after update
        updatedClocks.forEach(clock => {
            expect(clock.textContent.length).toBeGreaterThan(0, `Clock ${clock.id} should still have content after update`)
        })
        
        console.log('Clocks are rendering and updating correctly')
    })
})

