import './app.css';
import { mount, type MountState } from './shared';

const state: MountState = import.meta.hot?.data.state ?? { count: 0 };
let unmount = mount('vite', state);
let active = true;

if (import.meta.hot) {
  import.meta.hot.accept('./shared', (shared) => {
    if (!active || !shared) return;
    unmount();
    unmount = shared.mount('vite', state);
  });
  import.meta.hot.accept();
  import.meta.hot.dispose((data) => {
    data.state = state;
    active = false;
    unmount();
  });
}
