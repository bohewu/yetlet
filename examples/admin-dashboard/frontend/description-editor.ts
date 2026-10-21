import { Compartment, EditorState } from '@codemirror/state';
import { EditorView, keymap } from '@codemirror/view';
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';

export interface DescriptionEditor {
  connect(onEdit: (id: number, text: string) => void): void;
  update(id: number, text: string, readOnly: boolean, reset: number, invalid: boolean): void;
  destroy(): void;
}

const hintText = 'Plain text, up to 4,000 characters. Undo history resets on item switch or Reset.';
const length = (text: string) => Array.from(text).length;

export function createDescriptionEditor(host: HTMLElement, nonce: string): DescriptionEditor {
  if (!/^[a-f0-9]{32}$/.test(nonce)) throw new Error('Editor style nonce unavailable');
  const fallback = document.getElementById('triage-description');
  const label = document.getElementById('triage-description-label');
  const hint = document.getElementById('triage-description-hint');
  if (!(fallback instanceof HTMLTextAreaElement) || !label || !hint) throw new Error('Editor fallback unavailable');
  const access = new Compartment();
  let active = true, syncing = false, item = 0, readOnly = true, resetKey = 0;
  let onEdit: ((id: number, text: string) => void) | undefined;
  const permissions = (locked: boolean) => [EditorState.readOnly.of(locked), EditorView.editable.of(!locked)];
  const state = (text: string, locked: boolean) => EditorState.create({
    doc: text,
    extensions: [
      EditorView.cspNonce.of(nonce),
      EditorView.lineWrapping,
      history(), keymap.of([...historyKeymap, ...defaultKeymap]),
      access.of(permissions(locked)),
      EditorView.contentAttributes.of({ 'aria-labelledby': label.id, 'aria-describedby': hint.id }),
      EditorState.transactionFilter.of(transaction => {
        if (transaction.docChanged && length(transaction.newDoc.toString()) > 4000) {
          hint.textContent = 'Description limit reached (4,000 characters). Shorten the text before adding more.';
          return [];
        }
        return transaction;
      }),
      EditorView.updateListener.of(update => {
        if (update.docChanged && active && !syncing && !readOnly && onEdit) {
          hint.textContent = hintText;
          // The callback's Rust reducer also checks the selected item and lifetime.
          onEdit(item, update.state.doc.toString());
        }
      }),
    ],
  });
  let view: EditorView | undefined;
  const focusEditor = (event: Event) => {
    if (active && !host.hidden) { event.preventDefault(); view?.focus(); }
  };
  try {
    view = new EditorView({ parent: host, state: state('', true) });
    // CSP failures can be silent. Only hide the native control after real styles apply.
    if (![...document.querySelectorAll('style')].some(tag => tag.nonce === nonce && tag.sheet)
        || getComputedStyle(view.dom).position !== 'relative') throw new Error('Editor styles unavailable');
    label.addEventListener('click', focusEditor);
  } catch (error) {
    active = false;
    try { view?.destroy(); }
    finally { view?.dom.remove(); }
    throw error;
  }
  const editor = view;
  return {
    connect(callback) {
      if (active) onEdit = callback;
    },
    update(id, text, locked, reset, invalid) {
      if (!active) return;
      if (!Number.isSafeInteger(id) || id <= 0 || typeof text !== 'string' || length(text) > 4000 || typeof locked !== 'boolean'
          || !Number.isInteger(reset) || reset < 0 || reset > 0xffffffff) {
        throw new Error('Invalid description input');
      }
      syncing = true;
      try {
        // Equal graph commits preserve the live selection and history. A different
        // item or programmatic text (Reset) starts a fresh editing document.
        if (item !== id || resetKey !== reset || editor.state.doc.toString() !== text) {
          item = id; readOnly = locked; resetKey = reset;
          editor.setState(state(text, locked));
          hint.textContent = hintText;
        } else if (readOnly !== locked) {
          readOnly = locked;
          editor.dispatch({ effects: access.reconfigure(permissions(locked)) });
        }
        host.hidden = false;
        fallback.hidden = true;
        editor.contentDOM.setAttribute('aria-invalid', String(invalid));
        editor.contentDOM.setAttribute('aria-describedby', hint.id + (invalid ? ' triage-description-error' : ''));
        host.dataset.invalid = String(invalid);
      } finally { syncing = false; }
    },
    destroy() {
      if (!active) return;
      // Clear the JS entry point before Rust releases its owned Closure.
      active = false; onEdit = undefined;
      label.removeEventListener('click', focusEditor);
      try { editor.destroy(); }
      finally {
        editor.dom.remove(); host.hidden = true; fallback.hidden = false;
        delete host.dataset.invalid;
        hint.textContent = hintText;
      }
    },
  };
}
