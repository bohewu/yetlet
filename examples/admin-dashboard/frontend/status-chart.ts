import { BarController, BarElement, CategoryScale, Chart, LinearScale } from 'chart.js';

Chart.register(BarController, BarElement, CategoryScale, LinearScale);

export interface StatusChart {
  update(counts: Uint32Array): void;
  destroy(): void;
}

export function createStatusChart(host: HTMLElement): StatusChart {
  const canvas = document.createElement('canvas');
  canvas.setAttribute('aria-hidden', 'true');
  host.append(canvas);
  let chart: Chart<'bar', number[], string>;
  try {
    if (!canvas.getContext('2d')) throw new Error('Canvas unavailable');
    chart = new Chart(canvas, {
      type: 'bar',
      data: {
        labels: ['Open', 'Active', 'Blocked', 'Closed'],
        datasets: [{ data: [0, 0, 0, 0], backgroundColor: ['#2563eb', '#0e7490', '#b45309', '#64748b'], borderRadius: 4, maxBarThickness: 44 }],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        animation: false,
        events: [],
        scales: {
          x: { grid: { display: false } },
          y: { beginAtZero: true, suggestedMax: 1, ticks: { precision: 0 } },
        },
      },
    });
  } catch (error) {
    Chart.getChart(canvas)?.destroy();
    canvas.remove();
    throw error;
  }
  let active = true;
  let previous: number[] = [];
  return {
    update(counts) {
      if (!active) return;
      // The incoming typed array borrows WASM memory. Retain an owned copy only.
      const values = Array.from(counts);
      if (values.length !== 4 || values.some(value => !Number.isInteger(value) || value < 0 || value > 12) || values.reduce((sum, value) => sum + value, 0) > 12) {
        throw new Error('Invalid loaded-item counts');
      }
      if (values.every((value, index) => value === previous[index])) return;
      previous = values;
      host.hidden = false;
      chart.data.datasets[0].data = values;
      chart.update('none');
    },
    destroy() {
      if (!active) return;
      active = false;
      try { chart.destroy(); }
      finally {
        canvas.remove();
        if (!host.childElementCount) host.hidden = true;
      }
    },
  };
}
