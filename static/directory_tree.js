// static/directory_tree.js

import { playVideo } from './video_player.js';

function mixHue(h1, h2, t) {
    const d = ((((h2 - h1) % 360) + 540) % 360) - 180;
    return (h1 + d * t + 360) % 360;
}
function intensityToColor(v) {
    const val = Math.max(0, Math.min(100, Number(v) || 0));
    let hue;
    if (val <= 20) {
        hue = 120; // pure green up to 20
    } else if (val <= 40) {
        const t = (val - 20) / 20;
        hue = mixHue(120, 0, t);
    } else if (val <= 60) {
        const t = (val - 40) / 20;
        hue = mixHue(0, 330, t);
    } else if (val <= 80) {
        const t = (val - 60) / 20;
        hue = mixHue(330, 180, t);
    } else {
        hue = 180; // pure cyan for 80+
    }

    const S = 100; // fixed saturation
    const baseL = 50; // base lightness

    // brighten hues near red/orange (they look darker to the eye)
    function hueDistance(a, b) {
        let d = Math.abs(a - b) % 360;
        if (d > 180) d = 360 - d;
        return d;
    }
    const distToRed = hueDistance(hue, 0);
    const BOOST_MAX = 12; // max lightness boost (percentage points)
    const boost = distToRed <= 60 ? (1 - distToRed / 60) * BOOST_MAX : 0;
    const L = Math.min(90, baseL + boost);

    return `hsl(${hue.toFixed(1)}, ${S}%, ${L.toFixed(1)}%)`;
}

export function initDirectoryTree(
    directoryTreeData,
    containerElement,
    funscriptMap = {}
) {
    if (!directoryTreeData || !containerElement) {
        console.error('Directory tree data or container element is missing.');
        return;
    }

    containerElement.innerHTML = ''; // Clear previous content
    const rootUl = document.createElement('ul');
    rootUl.id = 'directory-tree-root';

    // helper: collect average_intensity values for all variants of a given video base path
    function collectVariantAverages(filePath) {
        const base = filePath.replace(/\.[^/.]+$/, ''); // remove extension
        const baseNorm = base.replace(/^\/+/, '');
        const avgs = [];
        for (const key in funscriptMap) {
            if (!Object.prototype.hasOwnProperty.call(funscriptMap, key))
                continue;
            if (!key) continue;
            const keyNorm = key.replace(/^\/+/, '');
            if (!keyNorm.endsWith('.funscript')) continue;
            // match either exact original (base.funscript) or variants base.<variant>.funscript
            if (
                keyNorm === `${baseNorm}.funscript` ||
                keyNorm.startsWith(`${baseNorm}.`)
            ) {
                const entry = funscriptMap[key];
                // Parenthesize to avoid mixing && and ?? which is a syntax error in JS
                const v = Number(
                    (entry &&
                        (entry['average_intensity'] ??
                            entry.average_intensity)) ??
                        NaN
                );
                if (isFinite(v)) avgs.push(v);
            }
        }
        return avgs;
    }

    // comparator: directories first, then files with funscripts (sorted by lowest avg intensity low->high), then others
    function compareNodes(a, b) {
        if (a.is_dir && !b.is_dir) return -1;
        if (!a.is_dir && b.is_dir) return 1;

        if (!a.is_dir && !b.is_dir) {
            const aAvgs = collectVariantAverages(a.path);
            const bAvgs = collectVariantAverages(b.path);
            const aHas = aAvgs.length > 0;
            const bHas = bAvgs.length > 0;
            if (aHas !== bHas) return aHas ? -1 : 1;
            if (aHas && bHas) {
                const ai = Math.min(...aAvgs);
                const bi = Math.min(...bAvgs);
                if (isFinite(ai) && isFinite(bi) && ai !== bi) return ai - bi;
            }
            return a.name.localeCompare(b.name);
        }

        return a.name.localeCompare(b.name);
    }

    function toggleFolder(id) {
        const element = document.getElementById(id);
        if (!element) {
            return;
        }

        // Get the parent <ul> of the clicked folder's <li>
        const parentUl = element.parentElement.parentElement;

        // Find all direct sibling <ul> elements and hide them.
        const siblingUls = parentUl.querySelectorAll(':scope > li > ul');
        siblingUls.forEach((ul) => {
            if (ul.id !== id) {
                ul.classList.add('hidden');
            }
        });

        // Toggle the visibility of the clicked folder's content
        element.classList.toggle('hidden');
    }

    function renderTree(node, parent) {
        const li = document.createElement('li');
        if (node.is_dir) {
            const folder = document.createElement('span');
            folder.textContent = node.name;
            folder.className = 'folder';
            folder.setAttribute('data-id', node.path);
            folder.onclick = () => toggleFolder(node.path);
            li.appendChild(folder);

            const ul = document.createElement('ul');
            ul.id = node.path;
            ul.className = 'hidden';

            const children = (node.children || []).slice();
            children
                .sort(compareNodes)
                .forEach((child) => renderTree(child, ul));
            li.appendChild(ul);
        } else {
            // gather variant stats
            const avgs = collectVariantAverages(node.path);

            const row = document.createElement('div');
            row.className = 'file-row';

            // only render a badge when there are funscsript averages
            if (avgs.length > 0) {
                const badge = document.createElement('span');
                badge.className = 'file-intensity';

                const unique = Array.from(new Set(avgs.map((v) => Number(v))))
                    .filter((v) => isFinite(v))
                    .sort((a, b) => a - b);

                if (unique.length > 0) {
                    // create stacked spans (no <br>) so CSS can handle layout
                    badge.innerHTML = unique
                        .map(
                            (v) =>
                                `<span style="color:${intensityToColor(
                                    v
                                )}">${v.toFixed(1)}</span>`
                        )
                        .join('');
                    row.appendChild(badge);
                }
            }

            const file = document.createElement('a');
            file.textContent = node.name;
            file.href = '#';
            file.onclick = (e) => {
                e.preventDefault();
                playVideo(
                    `/site/video/${node.path}`,
                    `/site/funscripts/${node.path.replace(/\.[^/.]+$/, '.funscript')}`
                );
            };
            row.appendChild(file);
            li.appendChild(row);
        }
        parent.appendChild(li);
    }

    const topChildren = (directoryTreeData.children || []).slice();
    topChildren
        .sort(compareNodes)
        .forEach((child) => renderTree(child, rootUl));
    containerElement.appendChild(rootUl);
}
