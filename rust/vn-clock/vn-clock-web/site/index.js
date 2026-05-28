import init, {WebApp} from '../pkg/vn_clock_web.js';

const HELP_TEXT = `
    SCROLLING:
    - Left/Right: Switch between Config Status and Event Log
    - Up/Down: Scroll selected section

    CLOCK CONTROLS:
    - Space: Pause/Resume
    - v: Set Speed (multiplier)
    - r: Reset to initial time
    - t: Set Time (HH:MM:SS)
    - e: Event Management Mode
    - S: Save Configuration (events + start time + speed)
    - L: Load Configuration
    - s: Save State (current time + events + speed)
    - l: Load State
    - h: Show/Hide Help

    EVENT MANAGEMENT:
    - a: Add new event
    - d: Delete selected event
    - Up/Down: Navigate list
    - Esc: Return to Clock view
`;

const InputMode = {
    Normal: 'Normal',
    InputFlow: 'Input Flow',
    EventManagement: 'Event Management',
    Help: 'Help Overlay',
    LoadingConfig: 'Loading Config',
    SavingConfig: 'Saving Config',
    LoadingState: 'Loading State',
    SavingState: 'Saving State'
};

const Section = {
    Config: 'Config',
    Log: 'Log'
};

async function start() {
    console.log("WebApp starting...");
    try {
        await init();
        console.log("WASM initialized");
        const app = new WebApp();
        console.log("WebApp instance created");

        // App state
        let inputMode = InputMode.Normal;
        let selectedSection = Section.Log;
        let selectedEvent = 0;
        let isProcessingAsync = false;

        // Temp state for adding event
        let tempEventName = "";
        let tempEventTime = null;
        let tempEventAutoPause = false;
        let tempEventRepeatInterval = null;

        let lastModeBeforeHelp = InputMode.Normal;

        // DOM Elements
        const clockDisplay = document.getElementById('clockTime');
        const statusText = document.getElementById('statusText');
        const configContent = document.getElementById('configContent');
        const logContent = document.getElementById('logContent');
        const configPanel = document.getElementById('configPanel');
        const logPanel = document.getElementById('logPanel');
        const helpOverlay = document.getElementById('helpOverlay');
        const helpText = document.getElementById('helpText');
        const inputOverlay = document.getElementById('inputOverlay');
        const mainInput = document.getElementById('mainInput');
        const inputTitle = document.getElementById('inputTitle');
        const configHeader = document.getElementById('configHeader');
        const logHeader = document.getElementById('logHeader');
        const helpButton = document.getElementById('helpButton');
        const tabClock = document.getElementById('tabClock');
        const tabEvents = document.getElementById('tabEvents');

        // Footer Buttons
        const btnSetTime = document.getElementById('btnSetTime');
        const btnSetSpeed = document.getElementById('btnSetSpeed');
        const btnReset = document.getElementById('btnReset');
        const btnSaveConfig = document.getElementById('btnSaveConfig');
        const btnLoadConfig = document.getElementById('btnLoadConfig');
        const btnSaveState = document.getElementById('btnSaveState');
        const btnLoadState = document.getElementById('btnLoadState');

        // Input Buttons
        const btnInputConfirm = document.getElementById('btnInputConfirm');
        const btnInputCancel = document.getElementById('btnInputCancel');

        helpText.textContent = HELP_TEXT;

        // Help overlay click behavior for mobile/touch or just convenience
        helpOverlay.addEventListener('click', (e) => {
            if (e.target === helpOverlay) {
                inputMode = lastModeBeforeHelp;
                helpOverlay.style.display = 'none';
            }
        });

        let audioCtx = null;

        function playDing() {
            try {
                if (!audioCtx) {
                    audioCtx = new (window.AudioContext || window.webkitAudioContext)();
                }
                if (audioCtx.state === 'suspended') {
                    audioCtx.resume();
                }

                const oscillator = audioCtx.createOscillator();
                const gainNode = audioCtx.createGain();

                oscillator.type = 'sine';
                oscillator.frequency.setValueAtTime(440, audioCtx.currentTime); // A4

                gainNode.gain.setValueAtTime(0.5, audioCtx.currentTime);
                gainNode.gain.exponentialRampToValueAtTime(0.0001, audioCtx.currentTime + 0.5);

                oscillator.connect(gainNode);
                gainNode.connect(audioCtx.destination);

                oscillator.start();
                oscillator.stop(audioCtx.currentTime + 0.5);
            } catch (e) {
                console.error("Failed to play sound:", e);
            }
        }

        async function update() {
            if (isProcessingAsync) {
                requestAnimationFrame(update);
                return;
            }

            if (inputMode === InputMode.InputFlow ||
                inputMode === InputMode.SavingConfig ||
                inputMode === InputMode.SavingState) {
                // Skip core tick during input overlay
                requestAnimationFrame(update);
                return;
            }

            try {
                app.tick();

                // Update Header
                const status = inputMode === InputMode.Normal ? (app.is_paused() ? "PAUSED" : "RUNNING") : inputMode.toUpperCase();
                clockDisplay.textContent = `${app.get_clock_time()} | ${status}`;

                // Update Footer
                statusText.textContent = `Mode: ${app.get_flow_name()}`;

                // Update Tabs
                tabClock.classList.toggle('active', inputMode === InputMode.Normal);
                tabEvents.classList.toggle('active', inputMode === InputMode.EventManagement);

                // Update Panels
                updateConfigPanel();
                updateLogPanel();

                // Mouse event listeners
                if (!window.hasInitializedMouseListeners) {
                    setupMouseListeners();
                }

                // Update Panel selection styles
                configPanel.classList.toggle('selected', selectedSection === Section.Config);
                logPanel.classList.toggle('selected', selectedSection === Section.Log);

                configHeader.textContent = selectedSection === Section.Config ? "Configuration Status (SELECTED)" : "Configuration Status";
                logHeader.textContent = selectedSection === Section.Log ? "Event Log (SELECTED)" : "Event Log";

                // Handle output events
                const outputEvents = await app.take_output_events();
                outputEvents.forEach(event => {
                    handleOutputEvent(event);
                });
            } catch (err) {
                console.error("Error in update loop:", err);
            }

            requestAnimationFrame(update);
        }

        function handleOutputEvent(event) {
            if (event === "Ding") {
                playDing();
            } else if (event.Log) {
                // UI could show a toast or something similar
            } else if (event.Paused !== undefined) {
                // UI reflects this in clockDisplay
            } else if (event.TimeSet) {
                // UI reflects this in clockDisplay
            } else if (event.SpeedSet) {
                // UI reflects this in configPanel
            }
        }

        function setupMouseListeners() {
            configHeader.addEventListener('click', () => {
                if (inputMode === InputMode.Normal || inputMode === InputMode.EventManagement) {
                    selectedSection = Section.Config;
                }
            });

            logHeader.addEventListener('click', () => {
                if (inputMode === InputMode.Normal || inputMode === InputMode.EventManagement) {
                    selectedSection = Section.Log;
                }
            });

            clockDisplay.addEventListener('click', () => {
                if (inputMode === InputMode.Normal || inputMode === InputMode.EventManagement) {
                    app.handle_event(JSON.stringify("TogglePause")).catch(console.error);
                }
            });

            statusText.addEventListener('click', () => {
                if (inputMode === InputMode.Normal || inputMode === InputMode.EventManagement) {
                    lastModeBeforeHelp = inputMode;
                    inputMode = InputMode.Help;
                    helpOverlay.style.display = 'flex';
                }
            });

            helpButton.addEventListener('click', () => {
                if (inputMode === InputMode.Normal || inputMode === InputMode.EventManagement) {
                    lastModeBeforeHelp = inputMode;
                    inputMode = InputMode.Help;
                    helpOverlay.style.display = 'flex';
                }
            });

            tabClock.addEventListener('click', () => {
                if (inputMode === InputMode.Normal || inputMode === InputMode.EventManagement) {
                    inputMode = InputMode.Normal;
                }
            });

            tabEvents.addEventListener('click', () => {
                if (inputMode === InputMode.Normal || inputMode === InputMode.EventManagement) {
                    inputMode = InputMode.EventManagement;
                    selectedEvent = 0;
                }
            });

            btnSetTime.addEventListener('click', () => {
                if (inputMode === InputMode.Normal) {
                    if (!app.is_paused()) app.handle_event(JSON.stringify("TogglePause")).catch(console.error);
                    startInput(InputMode.InputFlow, "SET TIME (HH:MM:SS)");
                }
            });

            btnSetSpeed.addEventListener('click', () => {
                if (inputMode === InputMode.Normal) {
                    startInput(InputMode.EditingSpeed, "SET SPEED (multiplier)");
                }
            });

            btnReset.addEventListener('click', () => {
                if (inputMode === InputMode.Normal) {
                    app.handle_event(JSON.stringify("Reset")).catch(console.error);
                }
            });

            btnSaveConfig.addEventListener('click', () => {
                if (inputMode === InputMode.Normal) {
                    startInput(InputMode.SavingConfig, "SAVE CONFIGURATION (Filename)");
                }
            });

            btnLoadConfig.addEventListener('click', () => {
                if (inputMode === InputMode.Normal) {
                    inputMode = InputMode.LoadingConfig;
                    isProcessingAsync = true;
                    app.load_config().finally(() => {
                        isProcessingAsync = false;
                        inputMode = InputMode.Normal;
                    });
                }
            });

            btnSaveState.addEventListener('click', () => {
                if (inputMode === InputMode.Normal) {
                    startInput(InputMode.SavingState, "SAVE STATE (Filename)");
                }
            });

            btnLoadState.addEventListener('click', () => {
                if (inputMode === InputMode.Normal) {
                    inputMode = InputMode.LoadingState;
                    isProcessingAsync = true;
                    app.load_state().finally(() => {
                        isProcessingAsync = false;
                        inputMode = InputMode.Normal;
                    });
                }
            });

            btnInputConfirm.addEventListener('click', () => {
                handleInputSubmit(mainInput.value);
            });

            btnInputCancel.addEventListener('click', () => {
                cancelInput();
            });

            window.hasInitializedMouseListeners = true;
        }

        async function updateConfigPanel() {
            // If we are in an input mode related to adding events, don't re-render the list
            // as it would re-create the buttons and might mess with click events or focus
            if (inputMode === InputMode.InputFlow ||
                inputMode === InputMode.SavingConfig ||
                inputMode === InputMode.SavingState) {
                return;
            }

            if (inputMode === InputMode.EventManagement) {
                // Only re-render if content is different or if it's the first time
                const events = await app.get_events();
                const currentEventCount = configContent.querySelectorAll('.list-item').length;
                const hasAddButton = configContent.querySelector('button') !== null;

                if (!hasAddButton || events.length !== currentEventCount) {
                    configContent.innerHTML = '';

                    const controlsDiv = document.createElement('div');
                    controlsDiv.style.marginBottom = '10px';
                    controlsDiv.style.display = 'flex';
                    controlsDiv.style.gap = '5px';

                    const btnAdd = document.createElement('button');
                    btnAdd.textContent = 'Add Event';
                    btnAdd.onclick = (event) => {
                        event.stopPropagation();
                        startInput(InputMode.InputFlow, "ADD EVENT: NAME");
                    };

                    controlsDiv.appendChild(btnAdd);
                    configContent.appendChild(controlsDiv);

                    events.forEach((e, i) => {
                        const div = document.createElement('div');
                        div.className = 'list-item';
                        if (i === selectedEvent) div.classList.add('selected');

                        div.textContent = `[${e.id}] ${e.display_string}`;
                        div.style.color = app.get_event_color_hex(e.id);
                        configContent.appendChild(div);

                        const btnDel = document.createElement('button');
                        btnDel.textContent = 'Delete';
                        btnDel.style.marginLeft = '10px';
                        btnDel.style.padding = '2px 5px';
                        btnDel.style.fontSize = '10px';
                        btnDel.onclick = async (ev) => {
                            ev.stopPropagation();
                            selectedEvent = i;
                            try {
                                await app.handle_event(JSON.stringify({RemoveTimedEvent: e.id}));
                            } catch (err) {
                                console.error("Failed to remove event:", err);
                            }
                        };
                        div.appendChild(btnDel);

                        div.addEventListener('click', () => {
                            selectedEvent = i;
                            // Force re-render to update selection style
                            updateConfigPanelSelection();
                        });

                        div.addEventListener('dblclick', async () => {
                            selectedEvent = i;
                            try {
                                await app.handle_event(JSON.stringify({RemoveTimedEvent: e.id}));
                            } catch (err) {
                                console.error("Failed to remove event:", err);
                            }
                        });
                    });
                } else {
                    updateConfigPanelSelection();
                }
            } else {
                let html = `<div>${app.get_initial_time_string()}</div>`;
                html += `<div>${app.get_target_speed_string()}</div>`;
                html += `<div>Events:</div>`;

                const events = app.get_events();
                if (events.length === 0) {
                    html += `<div>  (None)</div>`;
                } else {
                    events.forEach((e, i) => {
                        html += `<div style="color: ${app.get_event_color_hex(e.id)}">  [${e.id}] ${e.display_string}</div>`;
                    });
                }
                configContent.innerHTML = html;
            }
        }

        function updateConfigPanelSelection() {
            const items = configContent.querySelectorAll('.list-item');
            items.forEach((item, i) => {
                item.classList.toggle('selected', i === selectedEvent);
            });
        }

        async function updateLogPanel() {
            const logs = await app.get_logs();
            logContent.innerHTML = '';
            logs.slice().reverse().forEach(log => {
                const div = document.createElement('div');
                div.textContent = log.message;
                div.style.color = getColorCode(log.color);
                logContent.appendChild(div);
            });
        }

        function formatDuration(duration) {
            if (Array.isArray(duration)) {
                const total_secs = duration[0];
                const hours = Math.floor(total_secs / 3600);
                const minutes = Math.floor((total_secs % 3600) / 60);
                const seconds = total_secs % 60;
                return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
            }
            if (typeof duration === 'number') return `${duration}s`;
            if (duration && typeof duration.secs === 'number') {
                const total_secs = duration.secs;
                const hours = Math.floor(total_secs / 3600);
                const minutes = Math.floor((total_secs % 3600) / 60);
                const seconds = total_secs % 60;
                return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
            }
            // Handle direct second value if it's not wrapped in an object
            if (typeof duration === 'object' && duration !== null) {
                return JSON.stringify(duration);
            }
            return String(duration);
        }

        function getColorCode(color) {
            return app.get_color_hex(color);
        }

        // Keyboard Handling
        window.addEventListener('keydown', (e) => {
            if (isProcessingAsync) return;
            console.log("Key pressed:", e.key, "Mode:", inputMode);

            // Initialize audio context on first user interaction
            if (!audioCtx) {
                audioCtx = new (window.AudioContext || window.webkitAudioContext)();
            }
            if (audioCtx.state === 'suspended') {
                audioCtx.resume();
            }

            // If the user is typing in the input field, don't trigger global shortcuts
            if (document.activeElement === mainInput) return;

            if (inputMode === InputMode.Normal) {
                handleNormalInput(e);
            } else if (inputMode === InputMode.Help) {
                handleHelpInput(e);
            } else if (inputMode === InputMode.EventManagement) {
                handleEventManagementInput(e);
            } else {
                // All other modes are input modes handled by the overlay
                // but we need to listen for Esc to cancel
                if (e.key === 'Escape') {
                    cancelInput();
                }
            }
        });

        function handleNormalInput(e) {
            switch (e.key) {
                case ' ':
                    e.preventDefault();
                    app.handle_event(JSON.stringify("TogglePause"));
                    break;
                case 'v':
                    if (!app.is_paused()) app.handle_event(JSON.stringify("TogglePause"));
                    startInput(InputMode.InputFlow, "SET SPEED (multiplier)");
                    app.start_input("EditingSpeed");
                    break;
                case 'r':
                    app.handle_event(JSON.stringify("Reset"));
                    break;
                case 't':
                    if (!app.is_paused()) app.handle_event(JSON.stringify("TogglePause"));
                    startInput(InputMode.InputFlow, "SET TIME (HH:MM:SS)");
                    app.start_input("EditingTime");
                    break;
                case 'e':
                    inputMode = InputMode.EventManagement;
                    selectedEvent = 0;
                    break;
                case 'h':
                    lastModeBeforeHelp = inputMode;
                    inputMode = InputMode.Help;
                    helpOverlay.style.display = 'flex';
                    break;
                case 'S':
                    startInput(InputMode.SavingConfig, "SAVE CONFIGURATION (Filename)");
                    break;
                case 'L':
                    inputMode = InputMode.LoadingConfig;
                    isProcessingAsync = true;
                    app.load_config().finally(() => {
                        isProcessingAsync = false;
                        inputMode = InputMode.Normal;
                    });
                    break;
                case 's':
                    startInput(InputMode.SavingState, "SAVE STATE (Filename)");
                    break;
                case 'l':
                    inputMode = InputMode.LoadingState;
                    isProcessingAsync = true;
                    app.load_state().finally(() => {
                        isProcessingAsync = false;
                        inputMode = InputMode.Normal;
                    });
                    break;
                case 'ArrowLeft':
                    selectedSection = Section.Config;
                    break;
                case 'ArrowRight':
                    selectedSection = Section.Log;
                    break;
                case 'ArrowUp':
                    e.preventDefault();
                    scrollPanel(-1);
                    break;
                case 'ArrowDown':
                    e.preventDefault();
                    scrollPanel(1);
                    break;
            }
        }

        function handleHelpInput(e) {
            if (e.key === 'h' || e.key === 'Escape') {
                inputMode = lastModeBeforeHelp;
                helpOverlay.style.display = 'none';
            }
        }

        async function handleEventManagementInput(e) {
            switch (e.key) {
                case 'a':
                    startInput(InputMode.InputFlow, "ADD EVENT: NAME");
                    app.start_input("AddingEvent");
                    break;
                case 'd':
                    const events = await app.get_events();
                    if (events.length > 0 && selectedEvent < events.length) {
                        app.handle_event(JSON.stringify({RemoveTimedEvent: events[selectedEvent].id}));
                        if (selectedEvent >= events.length - 1 && events.length > 1) {
                            selectedEvent = events.length - 2;
                        }
                    }
                    break;
                case 'ArrowUp':
                    e.preventDefault();
                    if (selectedEvent > 0) selectedEvent--;
                    break;
                case 'ArrowDown':
                    e.preventDefault();
                    const evs = app.get_events();
                    if (selectedEvent < evs.length - 1) selectedEvent++;
                    break;
                case 'h':
                    lastModeBeforeHelp = inputMode;
                    inputMode = InputMode.Help;
                    helpOverlay.style.display = 'flex';
                    break;
                case 'Escape':
                    inputMode = InputMode.Normal;
                    break;
            }
        }

        function scrollPanel(dir) {
            const panel = selectedSection === Section.Config ? configContent : logContent;
            panel.scrollTop += dir * 20;
        }

        function startInput(mode, title) {
            inputMode = mode;
            // Mode mapping is now simplified since we only use generic flow start names
            let stepName = mode;
            if (mode === InputMode.InputFlow) {
                // If title was passed as generic prompt, we should update it after start_input
                // but for simplicity we keep the logic as is for now
            } else if (mode === InputMode.EditingTime) {
                stepName = "EditingTime";
            } else if (mode === InputMode.EditingSpeed) {
                stepName = "EditingSpeed";
            }

            if (mode === InputMode.InputFlow) {
                // For AddEvent we use start_input("AddingEvent")
                app.start_input("AddingEvent");
            } else {
                app.start_input(stepName);
            }
            inputTitle.textContent = title;
            mainInput.value = "";
            inputOverlay.style.display = 'flex';
            setTimeout(() => mainInput.focus(), 10);
        }

        function cancelInput() {
            app.cancel_input();
            if (tabEvents.classList.contains('active')) {
                inputMode = InputMode.EventManagement;
            } else {
                inputMode = InputMode.Normal;
            }
            inputOverlay.style.display = 'none';
            mainInput.blur();
        }

        mainInput.onkeydown = (e) => {
            if (e.key === 'Enter') {
                const val = mainInput.value.trim();
                handleInputSubmit(val);
            } else if (e.key === 'Escape') {
                cancelInput();
            }
        };

        async function handleInputSubmit(val) {
            try {
                const result = await app.handle_input(val);
                if (result.success) {
                    if (result.next_step === "None") {
                        if (inputMode === InputMode.InputFlow) {
                            if (tabEvents.classList.contains('active')) {
                                inputMode = InputMode.EventManagement;
                            } else {
                                inputMode = InputMode.Normal;
                            }
                        } else {
                            inputMode = InputMode.Normal;
                        }
                        inputOverlay.style.display = 'none';
                        mainInput.blur();
                    } else {
                        // Update title based on prompt from core
                        inputTitle.textContent = result.prompt;
                        mainInput.value = "";
                    }
                    return; // Succesfully handled by state machine
                } else if (result.error && result.next_step !== "None") {
                    inputTitle.textContent = `${result.prompt} (${result.error})`;
                    return;
                }
            } catch (err) {
                console.error("Failed to handle input:", err);
            }

            // Fallback for file saving which is not yet in InputFlowState
            if (inputMode === InputMode.SavingConfig) {
                if (val) {
                    const json = app.get_config_json();
                    if (json) {
                        const filename = val.endsWith(".clockcfg") ? val : `${val}.clockcfg`;
                        downloadFile(filename, json);
                    }
                }
                inputMode = InputMode.Normal;
                inputOverlay.style.display = 'none';
                mainInput.blur();
            } else if (inputMode === InputMode.SavingState) {
                if (val) {
                    const json = app.get_state_json();
                    if (json) {
                        const filename = val.endsWith(".clockstate") ? val : `${val}.clockstate`;
                        downloadFile(filename, json);
                    }
                }
                inputMode = InputMode.Normal;
                inputOverlay.style.display = 'none';
                mainInput.blur();
            }
        }

        function downloadFile(filename, content) {
            const blob = new Blob([content], {type: 'application/json'});
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = filename;
            document.body.appendChild(a);
            a.click();
            setTimeout(() => {
                document.body.removeChild(a);
                window.URL.revokeObjectURL(url);
            }, 0);
        }

        requestAnimationFrame(update);
    } catch (e) {
        console.error("Failed to start WebApp:", e);
    }
}

start();
