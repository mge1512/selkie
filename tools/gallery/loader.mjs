/**
 * Custom loader to intercept dompurify import and provide properly initialized version
 */
import { JSDOM } from 'jsdom';
import DOMPurifyFactory from 'dompurify';

// Create JSDOM
const dom = new JSDOM('<!DOCTYPE html><html><body></body></html>', {
  pretendToBeVisual: true,
});

// Create properly initialized DOMPurify
const purify = DOMPurifyFactory(dom.window);

// Export for mermaid to use
export default purify;
