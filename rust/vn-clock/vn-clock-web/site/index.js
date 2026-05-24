import init, { WebApp } from '../pkg/vn_clock_web.js';

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
    EditingTime: 'Editing Time',
    EditingSpeed: 'Editing Speed',
    EventManagement: 'Event Management',
    AddingEventName: 'Adding Event (Name)',
    AddingEventTime: 'Adding Event (Time)',
    AddingEventAutoPause: 'Adding Event (Auto-Pause)',
    AddingEventRepeatInterval: 'Adding Event (Repeat Interval)',
    AddingEventRepeatUntil: 'Adding Event (Repeat Until)',
    LoadingConfig: 'Loading Config',
    SavingConfig: 'Saving Config',
    LoadingState: 'Loading State',
    SavingState: 'Saving State',
    Help: 'Help Overlay'
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

            helpText.textContent = HELP_TEXT;

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
                    
                    gainNode.gain.setValueAtTime(0.2, audioCtx.currentTime);
                    gainNode.gain.exponentialRampToValueAtTime(0.0001, audioCtx.currentTime + 0.2);

                    oscillator.connect(gainNode);
                    gainNode.connect(audioCtx.destination);

                    oscillator.start();
                    oscillator.stop(audioCtx.currentTime + 0.2);
                } catch (e) {
                    console.error("Failed to play sound:", e);
                }
            }

            function update() {
                if (isProcessingAsync) {
                    requestAnimationFrame(update);
                    return;
                }
                app.tick();
                
                // Update Header
                const status = inputMode === InputMode.Normal ? (app.is_paused() ? "PAUSED" : "RUNNING") : inputMode.toUpperCase();
                clockDisplay.textContent = `${app.get_clock_time()} | ${status}`;

                // Update Footer
                statusText.textContent = `Mode: ${inputMode}`;

                // Update Panels
                updateConfigPanel();
                updateLogPanel();
                
                // Update Panel selection styles
                configPanel.classList.toggle('selected', selectedSection === Section.Config);
                logPanel.classList.toggle('selected', selectedSection === Section.Log);
                
                configHeader.textContent = selectedSection === Section.Config ? "Configuration Status (SELECTED)" : "Configuration Status";
                logHeader.textContent = selectedSection === Section.Log ? "Event Log (SELECTED)" : "Event Log";

                // Handle output events
                const outputEvents = app.take_output_events();
                outputEvents.forEach(event => {
                    if (event === "Ding") {
                        playDing();
                    }
                });

                requestAnimationFrame(update);
            }

            function updateConfigPanel() {
                if (inputMode === InputMode.EventManagement) {
                    const events = app.get_events();
                    configContent.innerHTML = '';
                    events.forEach((e, i) => {
                        const div = document.createElement('div');
                        div.className = 'list-item';
                        if (i === selectedEvent) div.classList.add('selected');
                        
                        let text = `[${i}] ${e.name} at ${e.time}`;
                        if (e.auto_pause) text += " (Auto-pause)";
                        if (e.repeat_interval) {
                            text += ` | Every ${formatDuration(e.repeat_interval)}`;
                            if (e.repeat_until) text += ` until ${e.repeat_until}`;
                        }
                        div.textContent = text;
                        div.style.color = getColorCode(e.color);
                        configContent.appendChild(div);
                    });
                } else {
                    let html = `<div>Initial Time: ${app.get_initial_time()}</div>`;
                    html += `<div>Target Speed: ${app.get_target_speed().toFixed(2)}x</div>`;
                    html += `<div>Events:</div>`;
                    
                    const events = app.get_events();
                    if (events.length === 0) {
                        html += `<div>  (None)</div>`;
                    } else {
                        events.forEach((e, i) => {
                            let text = `  [${i}] ${e.name} at ${e.time}`;
                            if (e.auto_pause) text += " (Auto-pause)";
                            if (e.repeat_interval) {
                                text += ` | Every ${formatDuration(e.repeat_interval)}`;
                                if (e.repeat_until) text += ` until ${e.repeat_until}`;
                            }
                            html += `<div style="color: ${getColorCode(e.color)}">${text}</div>`;
                        });
                    }
                    configContent.innerHTML = html;
                }
            }

            function updateLogPanel() {
                const logs = app.get_logs();
                logContent.innerHTML = '';
                logs.slice().reverse().forEach(log => {
                    const div = document.createElement('div');
                    div.textContent = log.message;
                    div.style.color = getColorCode(log.color);
                    logContent.appendChild(div);
                });
            }

            function formatDuration(duration) {
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
                const colorMap = {
                    'Black': '#000000',
                    'Red': '#ff0000',
                    'Green': '#00ff00',
                    'Yellow': '#ffff00',
                    'Blue': '#0000ff',
                    'Magenta': '#ff00ff',
                    'Cyan': '#00ffff',
                    'White': '#ffffff',
                    'Grey': '#808080',
                    'DarkGrey': '#404040',
                    'LightRed': '#ff8080',
                    'LightGreen': '#80ff80',
                    'LightYellow': '#ffff80',
                    'LightBlue': '#8080ff',
                    'LightMagenta': '#ff80ff',
                    'LightCyan': '#80ffff'
                };
                return colorMap[color] || '#ffffff';
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
                        startInput(InputMode.EditingSpeed, "SET SPEED (multiplier)");
                        break;
                    case 'r':
                        app.handle_event(JSON.stringify("Reset"));
                        break;
                    case 't':
                        if (!app.is_paused()) app.handle_event(JSON.stringify("TogglePause"));
                        startInput(InputMode.EditingTime, "SET TIME (HH:MM:SS)");
                        break;
                    case 'e':
                        inputMode = InputMode.EventManagement;
                        selectedEvent = 0;
                        break;
                    case 'h':
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
                    inputMode = InputMode.Normal;
                    helpOverlay.style.display = 'none';
                }
            }

            function handleEventManagementInput(e) {
                switch (e.key) {
                    case 'a':
                        startInput(InputMode.AddingEventName, "ADD EVENT: NAME");
                        break;
                    case 'd':
                        app.handle_event(JSON.stringify({ RemoveTimedEvent: selectedEvent }));
                        const events = app.get_events();
                        if (selectedEvent >= events.length && events.length > 0) {
                            selectedEvent = events.length - 1;
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
                inputTitle.textContent = title;
                mainInput.value = "";
                inputOverlay.style.display = 'flex';
                setTimeout(() => mainInput.focus(), 10);
            }

            function cancelInput() {
                if (inputMode === InputMode.AddingEventName || 
                    inputMode === InputMode.AddingEventTime ||
                    inputMode === InputMode.AddingEventAutoPause ||
                    inputMode === InputMode.AddingEventRepeatInterval ||
                    inputMode === InputMode.AddingEventRepeatUntil) {
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

            function handleInputSubmit(val) {
                switch (inputMode) {
                    case InputMode.EditingTime:
                        app.handle_event(JSON.stringify({ SetTime: val }));
                        inputMode = InputMode.Normal;
                        break;
                    case InputMode.EditingSpeed:
                        const speed = parseFloat(val);
                        if (!isNaN(speed)) {
                            app.handle_event(JSON.stringify({ SetSpeed: speed }));
                        }
                        inputMode = InputMode.Normal;
                        break;
                    case InputMode.SavingConfig:
                        if (val) {
                            const json = app.get_config_json();
                            if (json) {
                                const filename = val.endsWith(".clockcfg") ? val : `${val}.clockcfg`;
                                downloadFile(filename, json);
                            }
                        }
                        inputMode = InputMode.Normal;
                        break;
                    case InputMode.SavingState:
                        if (val) {
                            const json = app.get_state_json();
                            if (json) {
                                const filename = val.endsWith(".clockstate") ? val : `${val}.clockstate`;
                                downloadFile(filename, json);
                            }
                        }
                        inputMode = InputMode.Normal;
                        break;
                    case InputMode.AddingEventName:
                        tempEventName = val;
                        startInput(InputMode.AddingEventTime, "ADD EVENT: TIME (HH:MM:SS)");
                        return;
                    case InputMode.AddingEventTime:
                        tempEventTime = val;
                        startInput(InputMode.AddingEventAutoPause, "ADD EVENT: AUTO-PAUSE? (y/n)");
                        return;
                    case InputMode.AddingEventAutoPause:
                        tempEventAutoPause = val.toLowerCase() === 'y';
                        startInput(InputMode.AddingEventRepeatInterval, "ADD EVENT: REPEAT INTERVAL (HH:MM:SS, empty to skip)");
                        return;
                    case InputMode.AddingEventRepeatInterval:
                        if (val === "") {
                            tempEventRepeatInterval = null;
                            submitEvent();
                        } else {
                            tempEventRepeatInterval = val;
                            startInput(InputMode.AddingEventRepeatUntil, "ADD EVENT: REPEAT UNTIL (HH:MM:SS)");
                        }
                        return;
                    case InputMode.AddingEventRepeatUntil:
                        const repeatUntil = val || null;
                        submitEvent(repeatUntil);
                        return;
                }
                inputOverlay.style.display = 'none';
                mainInput.blur();
            }

            function submitEvent(repeatUntil = null) {
                // Simple random color selection similar to TUI app_impl.rs
                const colors = [
                    'Red', 'Green', 'Yellow', 'Blue', 'Magenta', 'Cyan',
                    'LightRed', 'LightGreen', 'LightYellow', 'LightBlue', 'LightMagenta', 'LightCyan'
                ];
                const events = app.get_events();
                const color = colors[events.length % colors.length];

                const event = {
                    time: tempEventTime,
                    name: tempEventName,
                    auto_pause: tempEventAutoPause,
                    repeat_interval: tempEventRepeatInterval ? parseInterval(tempEventRepeatInterval) : null,
                    repeat_until: repeatUntil,
                    color: color
                };
                app.handle_event(JSON.stringify({ AddTimedEvent: event }));
                inputMode = InputMode.EventManagement;
                inputOverlay.style.display = 'none';
                mainInput.blur();
            }

            function parseInterval(s) {
                const parts = s.split(':').map(Number);
                let secs = 0;
                if (parts.length === 3) {
                    secs = parts[0] * 3600 + parts[1] * 60 + parts[2];
                } else if (parts.length === 2) {
                    secs = parts[0] * 60 + parts[1];
                } else if (parts.length === 1) {
                    secs = parts[0];
                }
                return { secs: secs, nanos: 0 };
            }

            function downloadFile(filename, content) {
                const blob = new Blob([content], { type: 'application/json' });
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
