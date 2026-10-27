// Application-local draw-submission estimate, independent of physics time/RAF rate.
export function createFrameRateCounter() {
  let start = null, frames = 0, fps = null;
  function reset(timestamp = null) { start = timestamp; frames = 0; fps = null; }
  return {
    get fps() { return fps; },
    reset,
    record(timestamp) {
      if (!Number.isFinite(timestamp) || timestamp < 0) { reset(); return; }
      if (start === null || timestamp < start) { reset(timestamp); return; }
      frames++;
      const elapsed = timestamp - start;
      if (elapsed >= 1000) {
        fps = frames * 1000 / elapsed;
        start = timestamp;
        frames = 0;
      }
    },
  };
}
