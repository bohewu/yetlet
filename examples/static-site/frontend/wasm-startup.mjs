// Optional startup presentation. The caller owns async work and resource retirement.
const owners = new WeakMap();

/**
 * @param {{root: HTMLElement, status: HTMLElement, indicator?: HTMLElement,
 * controls?: HTMLFieldSetElement, messages?: {loading?: string, ready?: string, error?: string}}} options
 */
export function createStartupShell({ root, status, indicator, controls, messages = {} }) {
  if (owners.has(root)) throw new Error('Startup root already has an owner');
  const owner = {};
  owners.set(root, owner);
  let current = null;
  let phase = 'idle';
  const text = {
    loading: messages.loading ?? status.textContent ?? 'Loading…',
    ready: messages.ready ?? '',
    error: messages.error ?? 'Unable to start. Check the page instructions, then reload.',
  };
  function paint(next, message) {
    if (typeof message !== 'string') throw new TypeError('Startup message must be text');
    phase = next;
    root.dataset.startupState = next;
    root.setAttribute('aria-busy', String(next === 'loading'));
    status.textContent = message;
    status.hidden = message.length === 0;
    if (indicator) indicator.hidden = next !== 'loading';
    if (controls) controls.disabled = next !== 'ready';
  }
  const live = ticket => ticket !== null && current === ticket && owners.get(root) === owner && phase !== 'disposed';
  return {
    get state() { return phase; },
    begin(message = text.loading) {
      if (phase === 'disposed' || phase === 'loading') return null;
      const ticket = {};
      paint('loading', message);
      current = ticket;
      return ticket;
    },
    ready(ticket, message = text.ready) {
      if (!live(ticket) || phase !== 'loading') return false;
      paint('ready', message);
      return true;
    },
    fail(ticket, message = text.error) {
      if (!live(ticket) || !['loading', 'ready'].includes(phase)) return false;
      paint('error', message);
      return true;
    },
    cancel(ticket, message = '') {
      if (!live(ticket)) return false;
      current = null;
      paint('idle', message);
      return true;
    },
    dispose() {
      if (phase === 'disposed') return;
      current = null;
      paint('disposed', '');
      if (owners.get(root) === owner) owners.delete(root);
    },
  };
}
