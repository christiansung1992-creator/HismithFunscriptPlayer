// static/settings_menu.js

import { setAbsoluteMaximum, getAbsoluteMaximum, setVibrateMode } from './funscript_handler.js?v=245';

export function createSettingsMenu() {
    let settingsMenu = document.getElementById('settings-menu');
    if (!settingsMenu) {
        settingsMenu = document.createElement('div');
        settingsMenu.id = 'settings-menu';
        settingsMenu.style.position = 'absolute';
        settingsMenu.style.top = '60px';
        settingsMenu.style.right = '10px';
        settingsMenu.style.width = '250px';
        settingsMenu.style.backgroundColor = 'rgba(0, 0, 0, 0.8)';
        settingsMenu.style.color = 'white';
        settingsMenu.style.padding = '10px';
        settingsMenu.style.borderRadius = '5px';
        settingsMenu.style.boxShadow = '0 4px 8px rgba(0, 0, 0, 0.2)';
        settingsMenu.style.display = 'none'; // Hidden by default
        settingsMenu.style.zIndex = '10';

        // Add the loop toggle button
        const loopToggle = document.createElement('button');
        loopToggle.id = 'loop-toggle';
        loopToggle.textContent = 'Loop: Off';
        loopToggle.style.backgroundColor = 'rgb(70, 70, 70)';
        loopToggle.style.color = 'white';
        loopToggle.style.border = 'none';
        loopToggle.style.padding = '5px 10px';
        loopToggle.style.cursor = 'pointer';
        loopToggle.style.borderRadius = '3px';
        loopToggle.style.marginBottom = '10px';
        loopToggle.onclick = () => {
            const videoElement = document.querySelector('video');
            videoElement.loop = !videoElement.loop;
            loopToggle.textContent = `Loop: ${videoElement.loop ? 'On' : 'Off'}`;
        };
        settingsMenu.appendChild(loopToggle);

        // Add a Calibration button (replaces the old intensity multiplier UI)
        const calibrationButton = document.createElement('button');
        calibrationButton.id = 'calibration-button';
        calibrationButton.textContent = 'Calibration';
        calibrationButton.style.backgroundColor = 'rgb(70, 70, 70)';
        calibrationButton.style.color = 'white';
        calibrationButton.style.border = 'none';
        calibrationButton.style.padding = '5px 10px';
        calibrationButton.style.cursor = 'pointer';
        calibrationButton.style.borderRadius = '3px';
        calibrationButton.style.marginBottom = '10px';
        calibrationButton.onclick = () => {
            window.location.href = '/site/calibration';
        };
        settingsMenu.appendChild(calibrationButton);

        // Add the hard limit input field with lock/unlock
        const hardLimitInputLabel = document.createElement('label');
        hardLimitInputLabel.textContent = 'Max Intensity Limit: ';
        hardLimitInputLabel.style.display = 'block';
        hardLimitInputLabel.style.marginBottom = '5px';
        hardLimitInputLabel.style.pointerEvents = 'none'; // Ensure the button itself is clickable

        const hardLimitLockButton = document.createElement('button');
        hardLimitLockButton.id = 'hard-limit-lock-button';
        hardLimitLockButton.textContent = 'Unlock';
        hardLimitLockButton.style.backgroundColor = 'rgb(70, 70, 70)';
        hardLimitLockButton.style.color = 'white';
        hardLimitLockButton.style.border = 'none';
        hardLimitLockButton.style.padding = '5px 10px';
        hardLimitLockButton.style.cursor = 'pointer';
        hardLimitLockButton.style.borderRadius = '3px';
        hardLimitLockButton.style.marginBottom = '10px';
        hardLimitLockButton.style.pointerEvents = 'auto'; // Ensure the button itself is clickable

        const hardLimitInput = document.createElement('input');
        hardLimitInput.id = 'hard-limit-input';
        hardLimitInput.type = 'number';
        hardLimitInput.min = '0';
        hardLimitInput.max = '100';
        hardLimitInput.value = getAbsoluteMaximum().toString();
        hardLimitInput.style.width = '100%';
        hardLimitInput.disabled = true; // Initially disabled

        hardLimitLockButton.onclick = () => {
            if (hardLimitInput.disabled) {
                hardLimitInput.disabled = false;
                hardLimitLockButton.textContent = 'Lock';
                hardLimitInputLabel.style.pointerEvents = 'auto';
            } else {
                hardLimitInput.disabled = true;
                hardLimitLockButton.textContent = 'Unlock';
                hardLimitInputLabel.style.pointerEvents = 'none';
            }
        };

        hardLimitInput.onchange = () => {
            const value = parseInt(hardLimitInput.value, 10);
            if (value >= 0 && value <= 100) {
                setAbsoluteMaximum(value);
            } else {
                alert('Please enter a value between 0 and 100.');
                hardLimitInput.value = getAbsoluteMaximum().toString();
            }
        };

        hardLimitInputLabel.appendChild(hardLimitLockButton); // Add the lock button next to the label
        hardLimitInputLabel.appendChild(hardLimitInput); // Add the input field
        settingsMenu.appendChild(hardLimitInputLabel);

        const vibrateModeLabel = document.createElement('label');
        vibrateModeLabel.textContent = 'Vibrate Mode: ';
        vibrateModeLabel.style.display = 'block';
        vibrateModeLabel.style.marginBottom = '5px';

        const vibrateModeSelect = document.createElement('select');
        vibrateModeSelect.id = 'vibrate-mode-select';
        ['Rate', 'Beat'].forEach(mode => {
            const option = document.createElement('option');
            option.value = mode.toLowerCase();
            option.textContent = mode;
            vibrateModeSelect.appendChild(option);
        });
        vibrateModeSelect.value = 'Rate';

        vibrateModeSelect.onchange = () => {
            setVibrateMode(vibrateModeSelect.value);
        };

        settingsMenu.appendChild(vibrateModeLabel);
        settingsMenu.appendChild(vibrateModeSelect);

        document.body.appendChild(settingsMenu);
    }

    return settingsMenu;
}

export function toggleSettingsMenu() {
    const settingsMenu = document.getElementById('settings-menu');
    if (settingsMenu) {
        settingsMenu.style.display = settingsMenu.style.display === 'none' ? 'block' : 'none';
    }
}