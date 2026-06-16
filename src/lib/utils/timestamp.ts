const TIME_FORMAT: Intl.DateTimeFormatOptions = {
  hour: 'numeric',
  minute: '2-digit',
  second: '2-digit',
  hour12: true,
};

/** Parse epoch seconds/ms, ISO strings, or Date-like values into 12-hour local time. */
export function formatTimestamp(timestamp: unknown): string {
  if (timestamp == null || timestamp === '') return '—';

  if (timestamp instanceof Date) {
    return Number.isNaN(timestamp.getTime())
      ? '—'
      : timestamp.toLocaleTimeString(undefined, TIME_FORMAT);
  }

  const n = Number(timestamp);
  if (Number.isFinite(n) && n > 0) {
    const ms = n > 1_000_000_000_000 ? n : n * 1000;
    const d = new Date(ms);
    return Number.isNaN(d.getTime()) ? '—' : d.toLocaleTimeString(undefined, TIME_FORMAT);
  }

  const d = new Date(String(timestamp));
  return Number.isNaN(d.getTime()) ? '—' : d.toLocaleTimeString(undefined, TIME_FORMAT);
}
