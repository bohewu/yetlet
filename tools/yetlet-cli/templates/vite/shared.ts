import './shared.css';
import mark from './mark.svg';

export interface MountState {
  count: number;
}

let latestReveal: typeof import('./lazy').reveal | undefined;

if (import.meta.hot) {
  import.meta.hot.accept('./lazy', (lazy) => {
    latestReveal = lazy?.reveal;
  });
}

export function mount(profile: string, state: MountState = { count: 0 }): () => void {
  const button = document.querySelector<HTMLButtonElement>('#load-lazy');
  const result = document.querySelector<HTMLElement>('#lazy-result');
  const image = document.querySelector<HTMLImageElement>('#local-mark');
  if (!button || !result || !image) return () => {};
  if (image.getAttribute('src') !== mark) throw new Error('Manifest SVG mismatch');
  button.dataset.ready = profile;
  let active = true;
  const onClick = async () => {
    const { reveal } = await import('./lazy');
    // A click pending during replacement belongs to the disposed mount.
    if (!active) return;
    (latestReveal ?? reveal)(result, ++state.count);
  };
  button.addEventListener('click', onClick);
  return () => {
    active = false;
    button.removeEventListener('click', onClick);
  };
}
