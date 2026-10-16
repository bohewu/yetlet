// Optional application status only; never submits or replays a mutation.
for (const root of document.querySelectorAll('[data-board-stream]')) {
  const endpoint = root.getAttribute('data-board-stream');
  const message = root.querySelector('[data-stream-message]');
  const counts = document.getElementById('record-counts');
  const initial = Number(root.dataset.initialRevision);
  if (!/^\/board\/projects\/[12]\/events$/.test(endpoint || '') || !message || !counts || !Number.isSafeInteger(initial)) continue;
  let current = null, timer = null, retired = false, attempts = 0, epoch = 0;
  const events = new AbortController();
  const close = () => { epoch++; clearTimeout(timer); timer = null; current?.close(); current = null; };
  const connect = () => {
    if (retired || !root.isConnected) return;
    close();
    const generation = epoch;
    message.textContent = 'Connecting; the last stored snapshot remains visible.';
    const source = new EventSource(endpoint);
    current = source;
    const valid = () => !retired && root.isConnected && current === source && generation === epoch;
    const retry = () => {
      if (!valid()) return;
      close();
      message.textContent = attempts >= 5 ? 'Live status stopped after five retries. Refresh the records or reconnect explicitly.' : 'Live status unavailable; the displayed snapshot may be stale. Reconnecting...';
      if (attempts < 5) { timer = setTimeout(connect, Math.min(16000, 1000 * 2 ** attempts)); attempts++; }
    };
    source.addEventListener('snapshot', event => {
      if (!valid()) return;
      try {
        if (event.data.length > 1024) throw new Error('snapshot bound');
        const data = JSON.parse(event.data);
        const keys = ['revision','total','open','active','blocked','closed'];
        if (Object.keys(data).length !== keys.length || !keys.every(key => Number.isSafeInteger(data[key]) && data[key] >= 0) || data.total > 10000 || data.open + data.active + data.blocked + data.closed !== data.total) throw new Error('snapshot shape');
        // Send the live-region event before the adjacent count update; Orca
        // coalesces rapid text events. Unchanged snapshots need no DOM writes.
        const nextMessage = data.revision === initial ? 'Live status current. Records match the displayed revision.' : 'Status updated. Refresh authoritative records to see the latest work-item changes.';
        if (message.textContent !== nextMessage) message.textContent = nextMessage;
        const nextCounts = `${data.total} total · ${data.open} open · ${data.active} active · ${data.blocked} blocked · ${data.closed} closed`;
        if (counts.textContent !== nextCounts) counts.textContent = nextCounts;
        root.dataset.currentRevision = String(data.revision);
        root.dataset.streamState = 'current';
        attempts = 0;
      } catch { retry(); }
    });
    source.addEventListener('unavailable', retry);
    source.addEventListener('error', retry);
  };
  root.querySelector('[data-stream-stop]')?.addEventListener('click', () => { close(); message.textContent = 'Live status paused. The stored snapshot remains visible.'; root.dataset.streamState = 'paused'; }, {signal: events.signal});
  root.querySelector('[data-stream-retry]')?.addEventListener('click', () => { attempts = 0; connect(); }, {signal: events.signal});
  window.addEventListener('pagehide', () => { retired = true; close(); events.abort(); }, {once: true});
  connect();
}
