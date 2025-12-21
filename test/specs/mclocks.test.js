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

    it('should render multiple clocks with UTC, JST visible and Epoch hidden', async () => {
        // Connect to the application URL
        console.log('Connecting to http://localhost:1420...')
        await browser.url('/')

        // Wait for the application to initialize
        const mainElement = await $('#mclocks')
        await mainElement.waitForExist({ timeout: 30000 })

        // Wait for clock elements to be rendered
        await browser.waitUntil(
            async () => {
                const clockCount = await browser.execute(() => {
                    return document.querySelectorAll('[id^="mclk-"]').length
                })
                return clockCount >= 3 // UTC, JST, Epoch
            },
            {
                timeout: 30000,
                timeoutMsg: 'Clock elements were not rendered',
                interval: 1000
            }
        )

        // Get all clock elements with visibility information
        const clockInfo = await browser.execute(() => {
            const clocks = Array.from(document.querySelectorAll('[id^="mclk-"]'))
            const ul = document.querySelector('#mclocks ul')
            const listItems = ul ? Array.from(ul.querySelectorAll('li')) : []

            return {
                clockCount: clocks.length,
                clocks: clocks.map((el, index) => {
                    const li = el.closest('li')
                    // Clock name is the text node before the span element
                    // Get it from li's childNodes (first text node)
                    let clockName = null
                    if (li) {
                        // Find the first text node before the span
                        for (let i = 0; i < li.childNodes.length; i++) {
                            const node = li.childNodes[i]
                            if (node.nodeType === Node.TEXT_NODE) {
                                clockName = node.textContent.trim()
                                break
                            }
                            if (node === el) {
                                // If we reach the span before finding a text node,
                                // this might be a countdown clock or Epoch clock
                                break
                            }
                        }
                        // Fallback: if no text node found, check if it's Epoch clock
                        if (!clockName && index === 2) {
                            clockName = 'Epoch'
                        }
                    }

                    return {
                        id: el.id,
                        textContent: el.textContent.trim(),
                        hasContent: el.textContent.trim().length > 0,
                        name: clockName,
                        isVisible: li ? window.getComputedStyle(li).display !== 'none' : true,
                        parentDisplay: li ? window.getComputedStyle(li).display : 'unknown',
                        parentHidden: li ? li.hidden : false
                    }
                }),
                listItemCount: listItems.length
            }
        })

        console.log('Clock information:', JSON.stringify(clockInfo, null, 2))

        // Verify that exactly 3 clocks are rendered (UTC, JST, Epoch)
        expect(clockInfo.clockCount).toBe(3, 'Exactly 3 clocks should be rendered (UTC, JST, Epoch)')

        // Verify clock IDs follow the expected pattern (mclk-0, mclk-1, mclk-2)
        clockInfo.clocks.forEach((clock, index) => {
            expect(clock.id).toBe(`mclk-${index}`, `Clock ID should follow pattern mclk-${index}`)
        })

        // Verify UTC clock (mclk-0) is visible
        const utcClock = clockInfo.clocks[0]
        expect(utcClock.isVisible).toBe(true, 'UTC clock should be visible')
        expect(utcClock.hasContent).toBe(true, 'UTC clock should have content')
        expect(utcClock.name).toBe('UTC', 'First clock should be UTC')

        // Verify JST clock (mclk-1) is visible
        const jstClock = clockInfo.clocks[1]
        expect(jstClock.isVisible).toBe(true, 'JST clock should be visible')
        expect(jstClock.hasContent).toBe(true, 'JST clock should have content')
        expect(jstClock.name).toBe('JST', 'Second clock should be JST')

        // Verify Epoch clock (mclk-2) is hidden
        const epochClock = clockInfo.clocks[2]
        expect(epochClock.isVisible).toBe(false, 'Epoch clock should be hidden')
        expect(epochClock.hasContent).toBe(true, 'Epoch clock should have content (even if hidden)')

        // Verify that all clocks have content
        clockInfo.clocks.forEach(clock => {
            expect(clock.hasContent).toBe(true, `Clock ${clock.id} should have content`)
        })

        // Verify that clocks are rendered in list items
        expect(clockInfo.listItemCount).toBe(3, 'All 3 clocks should be in list items')

        console.log('Successfully verified: UTC and JST clocks are visible, Epoch clock is hidden')
    })

    it('should toggle Epoch time display with Ctrl+e and verify it updates', async () => {
        // Connect to the application URL
        console.log('Connecting to http://localhost:1420...')
        await browser.url('/')

        // Wait for the application to initialize
        const mainElement = await $('#mclocks')
        await mainElement.waitForExist({ timeout: 30000 })

        // Wait for clock elements to be rendered
        await browser.waitUntil(
            async () => {
                const clockCount = await browser.execute(() => {
                    return document.querySelectorAll('[id^="mclk-"]').length
                })
                return clockCount >= 3 // UTC, JST, Epoch
            },
            {
                timeout: 30000,
                timeoutMsg: 'Clock elements were not rendered',
                interval: 1000
            }
        )

        // Verify Epoch clock is initially hidden
        const initialEpochVisibility = await browser.execute(() => {
            const epochClock = document.querySelector('#mclk-2')
            if (!epochClock) return { visible: false, found: false }
            const li = epochClock.closest('li')
            return {
                found: true,
                visible: li ? window.getComputedStyle(li).display !== 'none' && !li.hidden : false
            }
        })

        expect(initialEpochVisibility.found).toBe(true, 'Epoch clock should exist')
        expect(initialEpochVisibility.visible).toBe(false, 'Epoch clock should be hidden initially')

        // Press Ctrl+e to toggle Epoch time display
        console.log('Pressing Ctrl+e to toggle Epoch time display...')
        await browser.keys(['Control', 'e'])

        // Wait for the Epoch clock to become visible
        await browser.waitUntil(
            async () => {
                const visibility = await browser.execute(() => {
                    const epochClock = document.querySelector('#mclk-2')
                    if (!epochClock) return false
                    const li = epochClock.closest('li')
                    return li ? window.getComputedStyle(li).display !== 'none' && !li.hidden : false
                })
                return visibility === true
            },
            {
                timeout: 5000,
                timeoutMsg: 'Epoch clock did not become visible after Ctrl+e',
                interval: 500
            }
        )

        // Get initial Epoch time value
        const initialEpochValue = await browser.execute(() => {
            const epochClock = document.querySelector('#mclk-2')
            return epochClock ? epochClock.textContent.trim() : null
        })

        expect(initialEpochValue).not.toBe(null, 'Epoch time value should exist')
        expect(initialEpochValue).toMatch(/^\d+$/, 'Epoch time should be a number')

        console.log(`Initial Epoch time value: ${initialEpochValue}`)

        // Wait for Epoch time to update (at least 2 seconds)
        console.log('Waiting for Epoch time to update...')
        await browser.pause(2500)

        // Get updated Epoch time value
        const updatedEpochValue = await browser.execute(() => {
            const epochClock = document.querySelector('#mclk-2')
            return epochClock ? epochClock.textContent.trim() : null
        })

        expect(updatedEpochValue).not.toBe(null, 'Updated Epoch time value should exist')
        expect(updatedEpochValue).toMatch(/^\d+$/, 'Updated Epoch time should be a number')

        console.log(`Updated Epoch time value: ${updatedEpochValue}`)

        // Verify that Epoch time has been updated (should be greater than initial value)
        const initialNum = parseInt(initialEpochValue, 10)
        const updatedNum = parseInt(updatedEpochValue, 10)
        expect(updatedNum).toBeGreaterThan(initialNum, 'Epoch time should have increased after waiting')

        // Verify Epoch clock is still visible
        const epochVisibility = await browser.execute(() => {
            const epochClock = document.querySelector('#mclk-2')
            if (!epochClock) return false
            const li = epochClock.closest('li')
            return li ? window.getComputedStyle(li).display !== 'none' && !li.hidden : false
        })

        expect(epochVisibility).toBe(true, 'Epoch clock should still be visible after update')

        console.log('Successfully verified: Epoch time is displayed and updating with Ctrl+e')
    })

    it('should toggle Epoch time display with Ctrl+u and verify it updates', async () => {
        // Connect to the application URL
        console.log('Connecting to http://localhost:1420...')
        await browser.url('/')

        // Wait for the application to initialize
        const mainElement = await $('#mclocks')
        await mainElement.waitForExist({ timeout: 30000 })

        // Wait for clock elements to be rendered
        await browser.waitUntil(
            async () => {
                const clockCount = await browser.execute(() => {
                    return document.querySelectorAll('[id^="mclk-"]').length
                })
                return clockCount >= 3 // UTC, JST, Epoch
            },
            {
                timeout: 30000,
                timeoutMsg: 'Clock elements were not rendered',
                interval: 1000
            }
        )

        // Verify Epoch clock is initially hidden
        const initialEpochVisibility = await browser.execute(() => {
            const epochClock = document.querySelector('#mclk-2')
            if (!epochClock) return { visible: false, found: false }
            const li = epochClock.closest('li')
            return {
                found: true,
                visible: li ? window.getComputedStyle(li).display !== 'none' && !li.hidden : false
            }
        })

        expect(initialEpochVisibility.found).toBe(true, 'Epoch clock should exist')
        expect(initialEpochVisibility.visible).toBe(false, 'Epoch clock should be hidden initially')

        // Press Ctrl+u to toggle Epoch time display
        console.log('Pressing Ctrl+u to toggle Epoch time display...')
        await browser.keys(['Control', 'u'])

        // Wait for the Epoch clock to become visible
        await browser.waitUntil(
            async () => {
                const visibility = await browser.execute(() => {
                    const epochClock = document.querySelector('#mclk-2')
                    if (!epochClock) return false
                    const li = epochClock.closest('li')
                    return li ? window.getComputedStyle(li).display !== 'none' && !li.hidden : false
                })
                return visibility === true
            },
            {
                timeout: 5000,
                timeoutMsg: 'Epoch clock did not become visible after Ctrl+u',
                interval: 500
            }
        )

        // Get initial Epoch time value
        const initialEpochValue = await browser.execute(() => {
            const epochClock = document.querySelector('#mclk-2')
            return epochClock ? epochClock.textContent.trim() : null
        })

        expect(initialEpochValue).not.toBe(null, 'Epoch time value should exist')
        expect(initialEpochValue).toMatch(/^\d+$/, 'Epoch time should be a number')

        console.log(`Initial Epoch time value: ${initialEpochValue}`)

        // Wait for Epoch time to update (at least 2 seconds)
        console.log('Waiting for Epoch time to update...')
        await browser.pause(2500)

        // Get updated Epoch time value
        const updatedEpochValue = await browser.execute(() => {
            const epochClock = document.querySelector('#mclk-2')
            return epochClock ? epochClock.textContent.trim() : null
        })

        expect(updatedEpochValue).not.toBe(null, 'Updated Epoch time value should exist')
        expect(updatedEpochValue).toMatch(/^\d+$/, 'Updated Epoch time should be a number')

        console.log(`Updated Epoch time value: ${updatedEpochValue}`)

        // Verify that Epoch time has been updated (should be greater than initial value)
        const initialNum = parseInt(initialEpochValue, 10)
        const updatedNum = parseInt(updatedEpochValue, 10)
        expect(updatedNum).toBeGreaterThan(initialNum, 'Epoch time should have increased after waiting')

        // Verify Epoch clock is still visible
        const epochVisibility = await browser.execute(() => {
            const epochClock = document.querySelector('#mclk-2')
            if (!epochClock) return false
            const li = epochClock.closest('li')
            return li ? window.getComputedStyle(li).display !== 'none' && !li.hidden : false
        })

        expect(epochVisibility).toBe(true, 'Epoch clock should still be visible after update')

        console.log('Successfully verified: Epoch time is displayed and updating with Ctrl+u')
    })

    it('should start 1-minute timer with Ctrl+1 and verify it updates', async () => {
        // Connect to the application URL
        console.log('Connecting to http://localhost:1420...')
        await browser.url('/')

        // Wait for the application to initialize
        const mainElement = await $('#mclocks')
        await mainElement.waitForExist({ timeout: 30000 })

        // Wait for initial clock elements to be rendered
        await browser.waitUntil(
            async () => {
                const clockCount = await browser.execute(() => {
                    return document.querySelectorAll('[id^="mclk-"]').length
                })
                return clockCount >= 3 // UTC, JST, Epoch
            },
            {
                timeout: 30000,
                timeoutMsg: 'Clock elements were not rendered',
                interval: 1000
            }
        )

        // Get initial clock count
        const initialClockCount = await browser.execute(() => {
            return document.querySelectorAll('[id^="mclk-"]').length
        })

        // Press Ctrl+1 to start 1-minute timer
        console.log('Pressing Ctrl+1 to start 1-minute timer...')
        await browser.keys(['Control', '1'])

        // Wait for timer clock to be added
        await browser.waitUntil(
            async () => {
                const clockCount = await browser.execute(() => {
                    return document.querySelectorAll('[id^="mclk-"]').length
                })
                return clockCount > initialClockCount
            },
            {
                timeout: 5000,
                timeoutMsg: 'Timer clock was not added after Ctrl+1',
                interval: 500
            }
        )

        // Get timer clock element (should be the last one)
        const timerClockInfo = await browser.execute(() => {
            const clocks = Array.from(document.querySelectorAll('[id^="mclk-"]'))
            const timerClock = clocks[clocks.length - 1]
            if (!timerClock) return null
            return {
                id: timerClock.id,
                textContent: timerClock.textContent.trim(),
                isVisible: timerClock.closest('li') ? window.getComputedStyle(timerClock.closest('li')).display !== 'none' : false
            }
        })

        expect(timerClockInfo).not.toBe(null, 'Timer clock should exist')
        expect(timerClockInfo.isVisible).toBe(true, 'Timer clock should be visible')
        // Timer format includes icon (e.g., "⏱01:00")
        expect(timerClockInfo.textContent).toMatch(/.*\d{2}:\d{2}$/, 'Timer should display in MM:SS format (with optional icon)')

        console.log(`Initial timer value: ${timerClockInfo.textContent}`)

        // Get initial timer value as seconds
        // Remove icon and other non-numeric characters before the time
        const parseTimerValue = (value) => {
            // Extract MM:SS pattern from the value
            const match = value.match(/(\d{2}):(\d{2})/)
            if (!match) return NaN
            const minutes = parseInt(match[1], 10)
            const seconds = parseInt(match[2], 10)
            return minutes * 60 + seconds
        }

        const initialTimerSeconds = parseTimerValue(timerClockInfo.textContent)
        expect(initialTimerSeconds).not.toBe(NaN, 'Initial timer value should be valid')
        expect(initialTimerSeconds).toBeGreaterThan(0, 'Timer should have positive value')
        expect(initialTimerSeconds).toBeLessThanOrEqual(60, '1-minute timer should be 60 seconds or less')

        // Wait for timer to update (at least 2 seconds)
        console.log('Waiting for timer to update...')
        await browser.pause(2500)

        // Get updated timer value
        const updatedTimerClockInfo = await browser.execute(() => {
            const clocks = Array.from(document.querySelectorAll('[id^="mclk-"]'))
            const timerClock = clocks[clocks.length - 1]
            if (!timerClock) return null
            return {
                id: timerClock.id,
                textContent: timerClock.textContent.trim()
            }
        })

        expect(updatedTimerClockInfo).not.toBe(null, 'Updated timer clock should exist')
        expect(updatedTimerClockInfo.id).toBe(timerClockInfo.id, 'Timer clock ID should remain the same')

        const updatedTimerSeconds = parseTimerValue(updatedTimerClockInfo.textContent)
        console.log(`Updated timer value: ${updatedTimerClockInfo.textContent} (${updatedTimerSeconds} seconds)`)

        // Verify that timer has been updated (should be less than initial value)
        expect(updatedTimerSeconds).not.toBe(NaN, 'Updated timer value should be valid')
        expect(updatedTimerSeconds).toBeLessThan(initialTimerSeconds, 'Timer should have decreased after waiting')

        console.log('Successfully verified: 1-minute timer is started and updating with Ctrl+1')
    })

    it('should pause timer with Ctrl+p', async () => {
        // Connect to the application URL
        console.log('Connecting to http://localhost:1420...')
        await browser.url('/')

        // Wait for the application to initialize
        const mainElement = await $('#mclocks')
        await mainElement.waitForExist({ timeout: 30000 })

        // Wait for initial clock elements to be rendered
        await browser.waitUntil(
            async () => {
                const clockCount = await browser.execute(() => {
                    return document.querySelectorAll('[id^="mclk-"]').length
                })
                return clockCount >= 3 // UTC, JST, Epoch
            },
            {
                timeout: 30000,
                timeoutMsg: 'Clock elements were not rendered',
                interval: 1000
            }
        )

        // Start a timer with Ctrl+1
        console.log('Pressing Ctrl+1 to start timer...')
        await browser.keys(['Control', '1'])

        // Wait for timer to be added
        await browser.waitUntil(
            async () => {
                const clockCount = await browser.execute(() => {
                    return document.querySelectorAll('[id^="mclk-"]').length
                })
                return clockCount >= 4 // UTC, JST, Epoch, Timer
            },
            {
                timeout: 5000,
                timeoutMsg: 'Timer clock was not added',
                interval: 500
            }
        )

        // Get timer clock element
        const getTimerValue = async () => {
            return await browser.execute(() => {
                const clocks = Array.from(document.querySelectorAll('[id^="mclk-"]'))
                const timerClock = clocks[clocks.length - 1]
                return timerClock ? timerClock.textContent.trim() : null
            })
        }

        // Wait a bit to ensure timer is running
        await browser.pause(1000)

        // Get timer value before pause
        const timerValueBeforePause = await getTimerValue()
        expect(timerValueBeforePause).not.toBe(null, 'Timer value should exist before pause')

        console.log(`Timer value before pause: ${timerValueBeforePause}`)

        // Press Ctrl+p to pause timer
        console.log('Pressing Ctrl+p to pause timer...')
        await browser.keys(['Control', 'p'])

        // Wait a bit for pause to take effect
        await browser.pause(1000)

        // Get timer value after pause (first check)
        const timerValueAfterPause1 = await getTimerValue()
        expect(timerValueAfterPause1).not.toBe(null, 'Timer value should exist after pause')

        console.log(`Timer value after pause (first check): ${timerValueAfterPause1}`)

        // Wait for 2 seconds while paused
        await browser.pause(2000)

        // Get timer value after waiting while paused (second check)
        const timerValueAfterPause2 = await getTimerValue()
        expect(timerValueAfterPause2).not.toBe(null, 'Timer value should exist while paused')

        console.log(`Timer value after pause (second check, after 2 seconds): ${timerValueAfterPause2}`)

        // Verify that timer value has not changed while paused
        // The value should remain the same after waiting 2 seconds
        expect(timerValueAfterPause2).toBe(timerValueAfterPause1, 'Timer should not update while paused')

        // Press Ctrl+p again to resume timer
        console.log('Pressing Ctrl+p again to resume timer...')
        await browser.keys(['Control', 'p'])

        // Wait a bit for resume to take effect
        await browser.pause(500)

        // Get timer value after resume
        const timerValueAfterResume = await getTimerValue()
        expect(timerValueAfterResume).not.toBe(null, 'Timer value should exist after resume')

        console.log(`Timer value after resume: ${timerValueAfterResume}`)

        // Wait for timer to update after resume
        await browser.pause(2000)

        // Get timer value after waiting while resumed
        const timerValueAfterResumeWait = await getTimerValue()
        expect(timerValueAfterResumeWait).not.toBe(null, 'Timer value should exist after resume wait')

        console.log(`Timer value after resume wait: ${timerValueAfterResumeWait}`)

        // Verify that timer value has changed after resume
        // Remove icon and other non-numeric characters before the time
        const parseTimerValue = (value) => {
            // Extract MM:SS pattern from the value
            const match = value.match(/(\d{2}):(\d{2})/)
            if (!match) return NaN
            const minutes = parseInt(match[1], 10)
            const seconds = parseInt(match[2], 10)
            return minutes * 60 + seconds
        }

        const resumeSeconds = parseTimerValue(timerValueAfterResume)
        const resumeWaitSeconds = parseTimerValue(timerValueAfterResumeWait)

        expect(resumeSeconds).not.toBe(NaN, 'Resume timer value should be valid')
        expect(resumeWaitSeconds).not.toBe(NaN, 'Resume wait timer value should be valid')
        expect(resumeWaitSeconds).toBeLessThan(resumeSeconds, 'Timer should update after resume')

        console.log('Successfully verified: Timer can be paused and resumed with Ctrl+p')
    })

    it('should remove timer with Ctrl+0 and Ctrl+Alt+0', async () => {
        // Connect to the application URL
        console.log('Connecting to http://localhost:1420...')
        await browser.url('/')

        // Wait for the application to initialize
        const mainElement = await $('#mclocks')
        await mainElement.waitForExist({ timeout: 30000 })

        // Wait for initial clock elements to be rendered
        await browser.waitUntil(
            async () => {
                const clockCount = await browser.execute(() => {
                    return document.querySelectorAll('[id^="mclk-"]').length
                })
                return clockCount >= 3 // UTC, JST, Epoch
            },
            {
                timeout: 30000,
                timeoutMsg: 'Clock elements were not rendered',
                interval: 1000
            }
        )

        // Start two timers
        console.log('Starting first timer with Ctrl+1...')
        await browser.keys(['Control', '1'])
        await browser.pause(500)

        console.log('Starting second timer with Ctrl+2...')
        await browser.keys(['Control', '2'])
        await browser.pause(500)

        // Wait for timers to be added
        await browser.waitUntil(
            async () => {
                const clockCount = await browser.execute(() => {
                    return document.querySelectorAll('[id^="mclk-"]').length
                })
                return clockCount >= 5 // UTC, JST, Epoch, Timer1, Timer2
            },
            {
                timeout: 5000,
                timeoutMsg: 'Timer clocks were not added',
                interval: 500
            }
        )

        // Get timer clock IDs before removal
        const timerIdsBefore = await browser.execute(() => {
            const clocks = Array.from(document.querySelectorAll('[id^="mclk-"]'))
            // Get last two clocks (should be timers)
            return clocks.slice(-2).map(clock => clock.id)
        })

        expect(timerIdsBefore.length).toBe(2, 'Should have 2 timers before removal')
        console.log(`Timer IDs before removal: ${timerIdsBefore.join(', ')}`)

        // Press Ctrl+0 to remove oldest timer (leftmost)
        console.log('Pressing Ctrl+0 to remove oldest timer...')
        await browser.keys(['Control', '0'])

        // Wait for timer to be removed
        await browser.pause(500)

        // Get timer clock IDs after removing oldest
        const timerIdsAfterRemoveOldest = await browser.execute(() => {
            const clocks = Array.from(document.querySelectorAll('[id^="mclk-"]'))
            // Get last clock (should be remaining timer)
            const lastClock = clocks[clocks.length - 1]
            return lastClock ? lastClock.id : null
        })

        expect(timerIdsAfterRemoveOldest).not.toBe(null, 'Should have remaining timer after removing oldest')
        expect(timerIdsAfterRemoveOldest).toBe(timerIdsBefore[1], 'Remaining timer should be the second timer (newest)')

        console.log(`Remaining timer ID after removing oldest: ${timerIdsAfterRemoveOldest}`)

        // Start another timer
        console.log('Starting another timer with Ctrl+3...')
        await browser.keys(['Control', '3'])
        await browser.pause(500)

        // Wait for new timer to be added
        await browser.waitUntil(
            async () => {
                const clockCount = await browser.execute(() => {
                    return document.querySelectorAll('[id^="mclk-"]').length
                })
                return clockCount >= 5 // UTC, JST, Epoch, Timer1, Timer2
            },
            {
                timeout: 5000,
                timeoutMsg: 'New timer clock was not added',
                interval: 500
            }
        )

        // Get timer clock IDs before removing newest
        const timerIdsBeforeRemoveNewest = await browser.execute(() => {
            const clocks = Array.from(document.querySelectorAll('[id^="mclk-"]'))
            // Get last two clocks (should be timers)
            return clocks.slice(-2).map(clock => clock.id)
        })

        expect(timerIdsBeforeRemoveNewest.length).toBe(2, 'Should have 2 timers before removing newest')
        console.log(`Timer IDs before removing newest: ${timerIdsBeforeRemoveNewest.join(', ')}`)

        // Press Ctrl+Alt+0 to remove newest timer (rightmost)
        console.log('Pressing Ctrl+Alt+0 to remove newest timer...')
        await browser.keys(['Control', 'Alt', '0'])

        // Wait for timer to be removed
        await browser.pause(500)

        // Get timer clock IDs after removing newest
        const timerIdsAfterRemoveNewest = await browser.execute(() => {
            const clocks = Array.from(document.querySelectorAll('[id^="mclk-"]'))
            // Get last clock (should be remaining timer)
            const lastClock = clocks[clocks.length - 1]
            return lastClock ? lastClock.id : null
        })

        expect(timerIdsAfterRemoveNewest).not.toBe(null, 'Should have remaining timer after removing newest')
        expect(timerIdsAfterRemoveNewest).toBe(timerIdsBeforeRemoveNewest[0], 'Remaining timer should be the first timer (oldest)')

        console.log(`Remaining timer ID after removing newest: ${timerIdsAfterRemoveNewest}`)

        // Verify that the removed timer is no longer in the DOM
        const removedTimerExists = await browser.execute((removedId) => {
            return document.getElementById(removedId) !== null
        }, timerIdsBeforeRemoveNewest[1])

        expect(removedTimerExists).toBe(false, 'Removed timer should not exist in DOM')

        console.log('Successfully verified: Timers can be removed with Ctrl+0 and Ctrl+Alt+0')
    })

    it('should start 90-minute timer with Ctrl+Alt+9 and verify it updates', async () => {
        // Connect to the application URL
        console.log('Connecting to http://localhost:1420...')
        await browser.url('/')

        // Wait for the application to initialize
        const mainElement = await $('#mclocks')
        await mainElement.waitForExist({ timeout: 30000 })

        // Wait for initial clock elements to be rendered
        await browser.waitUntil(
            async () => {
                const clockCount = await browser.execute(() => {
                    return document.querySelectorAll('[id^="mclk-"]').length
                })
                return clockCount >= 3 // UTC, JST, Epoch
            },
            {
                timeout: 30000,
                timeoutMsg: 'Clock elements were not rendered',
                interval: 1000
            }
        )

        // Get initial clock count
        const initialClockCount = await browser.execute(() => {
            return document.querySelectorAll('[id^="mclk-"]').length
        })

        // Press Ctrl+Alt+9 to start 90-minute timer (10 minutes × 9)
        console.log('Pressing Ctrl+Alt+9 to start 90-minute timer...')
        await browser.keys(['Control', 'Alt', '9'])

        // Wait for timer clock to be added
        await browser.waitUntil(
            async () => {
                const clockCount = await browser.execute(() => {
                    return document.querySelectorAll('[id^="mclk-"]').length
                })
                return clockCount > initialClockCount
            },
            {
                timeout: 5000,
                timeoutMsg: 'Timer clock was not added after Ctrl+Alt+9',
                interval: 500
            }
        )

        // Get timer clock element (should be the last one)
        const timerClockInfo = await browser.execute(() => {
            const clocks = Array.from(document.querySelectorAll('[id^="mclk-"]'))
            const timerClock = clocks[clocks.length - 1]
            if (!timerClock) return null
            return {
                id: timerClock.id,
                textContent: timerClock.textContent.trim(),
                isVisible: timerClock.closest('li') ? window.getComputedStyle(timerClock.closest('li')).display !== 'none' : false
            }
        })

        expect(timerClockInfo).not.toBe(null, 'Timer clock should exist')
        expect(timerClockInfo.isVisible).toBe(true, 'Timer clock should be visible')
        // Timer format includes icon (e.g., "⏱90:00")
        expect(timerClockInfo.textContent).toMatch(/.*\d{2}:\d{2}$/, 'Timer should display in MM:SS format (with optional icon)')

        console.log(`Initial timer value: ${timerClockInfo.textContent}`)

        // Get initial timer value as seconds
        // Remove icon and other non-numeric characters before the time
        const parseTimerValue = (value) => {
            // Extract MM:SS pattern from the value
            const match = value.match(/(\d{2}):(\d{2})/)
            if (!match) return NaN
            const minutes = parseInt(match[1], 10)
            const seconds = parseInt(match[2], 10)
            return minutes * 60 + seconds
        }

        const initialTimerSeconds = parseTimerValue(timerClockInfo.textContent)
        expect(initialTimerSeconds).not.toBe(NaN, 'Initial timer value should be valid')
        expect(initialTimerSeconds).toBeGreaterThan(0, 'Timer should have positive value')
        // 90 minutes = 5400 seconds, but timer might show 89:59 or 90:00 depending on timing
        expect(initialTimerSeconds).toBeGreaterThan(5300, '90-minute timer should be around 90 minutes (5400 seconds)')
        expect(initialTimerSeconds).toBeLessThanOrEqual(5400, '90-minute timer should be 5400 seconds or less')

        // Wait for timer to update (at least 2 seconds)
        console.log('Waiting for timer to update...')
        await browser.pause(2500)

        // Get updated timer value
        const updatedTimerClockInfo = await browser.execute(() => {
            const clocks = Array.from(document.querySelectorAll('[id^="mclk-"]'))
            const timerClock = clocks[clocks.length - 1]
            if (!timerClock) return null
            return {
                id: timerClock.id,
                textContent: timerClock.textContent.trim()
            }
        })

        expect(updatedTimerClockInfo).not.toBe(null, 'Updated timer clock should exist')
        expect(updatedTimerClockInfo.id).toBe(timerClockInfo.id, 'Timer clock ID should remain the same')

        const updatedTimerSeconds = parseTimerValue(updatedTimerClockInfo.textContent)
        console.log(`Updated timer value: ${updatedTimerClockInfo.textContent} (${updatedTimerSeconds} seconds)`)

        // Verify that timer has been updated (should be less than initial value)
        expect(updatedTimerSeconds).not.toBe(NaN, 'Updated timer value should be valid')
        expect(updatedTimerSeconds).toBeLessThan(initialTimerSeconds, 'Timer should have decreased after waiting')

        console.log('Successfully verified: 90-minute timer is started and updating with Ctrl+Alt+9')
    })
})
