import './shared.css';
import mark from './mark.svg';

export function mount(profile: string): void {
  const button = document.querySelector<HTMLButtonElement>('#load-lazy');
  const result = document.querySelector<HTMLElement>('#lazy-result');
  const image = document.querySelector<HTMLImageElement>('#local-mark');
  if (!button || !result || !image) throw new Error('Missing SSR elements');
  if (image.getAttribute('src') !== mark) throw new Error('Manifest SVG mismatch');
  button.dataset.ready = profile;
  let count = 0;
  button.addEventListener('click', async () => {
    const { reveal } = await import('./lazy');
    reveal(result, ++count);
  });
}
