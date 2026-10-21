import './lazy.css';

export function reveal(element: HTMLElement, count: number): void {
  element.textContent = `Lazy module loaded: ${count}`;
  element.classList.add('lazy-loaded');
}
