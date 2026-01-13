/**
 * DOM setup - must be imported before mermaid
 */
import { JSDOM } from 'jsdom';
import DOMPurify from 'dompurify';

// Set up JSDOM environment
const dom = new JSDOM('<!DOCTYPE html><html><body><div id="container"></div></body></html>', {
  pretendToBeVisual: true,
});

// Use Object.defineProperty for read-only globals
Object.defineProperty(global, 'window', { value: dom.window, writable: true });
Object.defineProperty(global, 'document', { value: dom.window.document, writable: true });
Object.defineProperty(global, 'navigator', { value: dom.window.navigator, writable: true, configurable: true });

// Create DOMPurify instance for JSDOM's window
const purify = DOMPurify(dom.window);

// Create a wrapper that works both as a function and object with sanitize method
const domPurifyWrapper = {
  sanitize: (html, options) => purify.sanitize(html, options),
  addHook: (...args) => purify.addHook(...args),
  removeHook: (...args) => purify.removeHook(...args),
  removeHooks: (hookName) => purify.removeHooks ? purify.removeHooks(hookName) : undefined,
  removeAllHooks: () => purify.removeAllHooks ? purify.removeAllHooks() : undefined,
  isSupported: true,
  version: purify.version || '3.0.0',
};

// Make it callable as a function too
const domPurifyCallable = function(html, options) {
  return purify.sanitize(html, options);
};
Object.assign(domPurifyCallable, domPurifyWrapper);

global.DOMPurify = domPurifyCallable;
dom.window.DOMPurify = domPurifyCallable;

// Mock SVG methods that JSDOM doesn't support
const mockBBox = { x: 0, y: 0, width: 100, height: 20, top: 0, bottom: 20, left: 0, right: 100 };

const originalCreateElementNS = document.createElementNS.bind(document);
document.createElementNS = function(namespaceURI, qualifiedName) {
  const element = originalCreateElementNS(namespaceURI, qualifiedName);
  if (namespaceURI === 'http://www.w3.org/2000/svg') {
    element.getBBox = () => mockBBox;
    element.getComputedTextLength = () => mockBBox.width;
    element.getBoundingClientRect = () => mockBBox;
    element.getScreenCTM = () => ({ a: 1, b: 0, c: 0, d: 1, e: 0, f: 0 });
    element.createSVGPoint = () => ({ x: 0, y: 0, matrixTransform: () => ({ x: 0, y: 0 }) });
  }
  return element;
};

export { dom, purify };
