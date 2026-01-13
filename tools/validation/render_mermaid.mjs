#!/usr/bin/env node
/**
 * Mermaid diagram renderer using the official mermaid library.
 *
 * Usage:
 *   node render_mermaid.mjs <diagram_text>
 *   echo "flowchart LR\nA-->B" | node render_mermaid.mjs -
 *   node render_mermaid.mjs --analyze - < diagram.mmd
 *
 * Output:
 *   Default: Raw SVG string
 *   --analyze: JSON with structural analysis
 *     {
 *       "svg": "<svg>...</svg>",
 *       "structure": {
 *         "width": 400,
 *         "height": 200,
 *         "nodeCount": 3,
 *         "edgeCount": 2,
 *         "labels": ["A", "B", "C"],
 *         "shapes": { "rect": 2, "polygon": 1 }
 *       }
 *     }
 */

import { readFileSync } from 'fs';
import { JSDOM } from 'jsdom';

// Set up minimal DOM environment for mermaid
const dom = new JSDOM('<!DOCTYPE html><html><body><div id="mermaid"></div></body></html>', {
  pretendToBeVisual: true,
});
global.window = dom.window;
global.document = dom.window.document;
Object.defineProperty(global, 'navigator', {
  value: dom.window.navigator,
  writable: true,
});

// Mock SVG methods that JSDOM doesn't support
const mockBBox = { x: 0, y: 0, width: 100, height: 20 };

// Polyfill getBBox for SVG elements
const originalCreateElementNS = document.createElementNS.bind(document);
document.createElementNS = function(namespaceURI, qualifiedName) {
  const element = originalCreateElementNS(namespaceURI, qualifiedName);
  if (namespaceURI === 'http://www.w3.org/2000/svg') {
    element.getBBox = () => mockBBox;
    element.getComputedTextLength = () => mockBBox.width;
    element.getBoundingClientRect = () => ({
      ...mockBBox,
      top: 0, left: 0, right: mockBBox.width, bottom: mockBBox.height,
      toJSON: () => mockBBox,
    });
  }
  return element;
};

// Also mock on prototype for elements created differently
if (typeof SVGElement !== 'undefined') {
  SVGElement.prototype.getBBox = () => mockBBox;
  SVGElement.prototype.getComputedTextLength = () => mockBBox.width;
}

// Mock window methods
global.window.getComputedStyle = () => ({
  getPropertyValue: () => '',
});

// Import mermaid after setting up DOM
const mermaid = (await import('mermaid')).default;

// Initialize mermaid with consistent settings
mermaid.initialize({
  startOnLoad: false,
  securityLevel: 'loose',
  flowchart: {
    htmlLabels: false,  // Use SVG text for easier comparison
    useMaxWidth: false,
  },
  theme: 'default',
});

/**
 * Analyze SVG structure for comparison
 */
function analyzeSvg(svgString) {
  const svgDom = new JSDOM(svgString, { contentType: 'image/svg+xml' });
  const svg = svgDom.window.document.querySelector('svg');

  if (!svg) {
    return null;
  }

  // Get dimensions
  const viewBox = svg.getAttribute('viewBox');
  let width = 0, height = 0;
  if (viewBox) {
    const parts = viewBox.split(/\s+/).map(Number);
    if (parts.length >= 4) {
      width = parts[2];
      height = parts[3];
    }
  }
  if (!width) width = parseFloat(svg.getAttribute('width')) || 0;
  if (!height) height = parseFloat(svg.getAttribute('height')) || 0;

  // Count shape elements
  const shapes = {
    rect: svg.querySelectorAll('rect').length,
    circle: svg.querySelectorAll('circle').length,
    ellipse: svg.querySelectorAll('ellipse').length,
    polygon: svg.querySelectorAll('polygon').length,
    path: svg.querySelectorAll('path').length,
    line: svg.querySelectorAll('line').length,
    polyline: svg.querySelectorAll('polyline').length,
  };

  // Count nodes (elements with .node class or data-node attribute)
  const nodeElements = svg.querySelectorAll('.node, .flowchart-node, [data-node]');
  const nodeCount = nodeElements.length;

  // Count edges (elements with .edge class or edge-related classes)
  const edgeElements = svg.querySelectorAll('.edge, .flowchart-link, .edgePath, [data-edge]');
  const edgeCount = edgeElements.length;

  // Extract text labels
  const textElements = svg.querySelectorAll('text, tspan');
  const labels = [];
  const seenLabels = new Set();
  textElements.forEach(el => {
    const text = el.textContent?.trim();
    if (text && !seenLabels.has(text)) {
      seenLabels.add(text);
      labels.push(text);
    }
  });

  // Extract node IDs
  const nodeIds = [];
  nodeElements.forEach(el => {
    const id = el.getAttribute('id') || el.getAttribute('data-id');
    if (id) nodeIds.push(id);
  });

  // Check for markers (arrows)
  const markers = svg.querySelectorAll('marker');
  const markerCount = markers.length;

  // Check for style/defs
  const hasDefs = svg.querySelector('defs') !== null;
  const hasStyle = svg.querySelector('style') !== null;

  return {
    width: Math.round(width),
    height: Math.round(height),
    nodeCount,
    edgeCount,
    labels: labels.sort(),
    nodeIds: nodeIds.sort(),
    shapes,
    markerCount,
    hasDefs,
    hasStyle,
  };
}

async function renderDiagram(text, analyze = false) {
  try {
    // Render the diagram
    const { svg } = await mermaid.render('mermaid-diagram', text);

    if (analyze) {
      const structure = analyzeSvg(svg);
      return {
        success: true,
        svg,
        structure,
      };
    }

    return {
      success: true,
      svg,
    };
  } catch (error) {
    return {
      success: false,
      error: error.message || String(error),
    };
  }
}

async function main() {
  const args = process.argv.slice(2);
  let input;
  let analyze = false;

  // Parse arguments
  const positionalArgs = [];
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--analyze' || args[i] === '-a') {
      analyze = true;
    } else {
      positionalArgs.push(args[i]);
    }
  }

  if (positionalArgs.length === 0) {
    console.error('Usage: node render_mermaid.mjs [--analyze] <diagram_text_or_file_or_-_for_stdin>');
    process.exit(1);
  }

  const inputArg = positionalArgs[0];

  if (inputArg === '-') {
    // Read from stdin
    input = readFileSync(0, 'utf-8');
  } else {
    // Try to read as file, fall back to treating as diagram text
    try {
      input = readFileSync(inputArg, 'utf-8');
    } catch {
      input = inputArg;
    }
  }

  const result = await renderDiagram(input, analyze);

  if (analyze || !result.success) {
    // Output as JSON for structured data or errors
    console.log(JSON.stringify(result, null, 2));
  } else {
    // Output raw SVG for simple rendering
    console.log(result.svg);
  }
}

main();
