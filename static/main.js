// static/main.js

import { initDirectoryTree } from './directory_tree.js';
import { initWebSocket } from './socket.js';
import { createSettingsMenu, toggleSettingsMenu } from './settings_menu.js';

function initializeUI() {
    // Create UI components that are globally available but may be hidden initially
    createSettingsMenu();

    const settingsBtn = document.getElementById('settings-button');
    settingsBtn.onclick = toggleSettingsMenu;
    settingsBtn.style.right = '10px';
    settingsBtn.style.display = 'block';
    settingsBtn.classList.add('player-button', 'btn');

    // Event listener for the sidebar toggle
    document.getElementById('toggle-directory').onclick = () => {
        document
            .getElementById('directory-container')
            .classList.toggle('hidden');
    };
}

async function main() {
    // Initialize core components
    initializeUI();
    initWebSocket();

    // Fetch directory tree and render it
    try {
        const response = await fetch('/api/directory-tree');
        if (!response.ok) {
            throw new Error(
                `Failed to fetch directory tree: ${response.statusText}`
            );
        }
        const payload = await response.json();
        const directoryTreeData = payload.tree || payload;
        const funscriptMap = payload.funscripts || {}; // pass funscript cache info
        const directoryTreeContainer =
            document.getElementById('directory-tree');

        try {
            if (payload && payload.funscript_cache_error) {
                const msg = payload.funscript_cache_error;
                const container = document.getElementById(
                    'directory-container'
                );
                if (container) {
                    let errEl = document.getElementById(
                        'funscript-cache-error'
                    );
                    if (!errEl) {
                        errEl = document.createElement('div');
                        errEl.id = 'funscript-cache-error';
                        errEl.style.cssText =
                            'color:#fff;background:#8b0000;padding:8px;border-radius:4px;margin:8px;font-weight:600;';
                        container.insertBefore(errEl, directoryTreeContainer);
                    }
                    errEl.textContent = msg;
                }
            } else {
                const existing = document.getElementById(
                    'funscript-cache-error'
                );
                if (existing) existing.remove();
            }
        } catch (e) {
            console.error('Error rendering funscript cache warning:', e);
        }

        initDirectoryTree(
            directoryTreeData,
            directoryTreeContainer,
            funscriptMap
        );
    } catch (error) {
        console.error(error);
        document.getElementById('directory-tree').innerHTML =
            `<p style="color: red; padding: 10px;">Error loading directory.</p>`;
    }
}

// Run the main application logic when the DOM is ready
document.addEventListener('DOMContentLoaded', main);
