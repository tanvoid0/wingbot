<script lang="ts">
  import { getImpactTrendSeries, summarizeTrendSeries } from '$lib/utils/matchHistory';

  interface Props {
    matches: unknown[];
    compact?: boolean;
  }

  let { matches, compact = false }: Props = $props();

  const series = $derived(getImpactTrendSeries(matches));
  const stats = $derived(summarizeTrendSeries(series));

  const width = 240;
  const height = $derived(compact ? 64 : 80);
  const padX = 6;
  const padY = 8;

  const chart = $derived.by(() => {
    if (series.length < 2) return null;

    const innerW = width - padX * 2;
    const innerH = height - padY * 2;
    const minV = Math.min(...series.map((p) => p.value), 0);
    const maxV = Math.max(...series.map((p) => p.value), 100);
    const range = Math.max(maxV - minV, 1);

    const points = series.map((p, i) => {
      const x = padX + (i / (series.length - 1)) * innerW;
      const y = padY + innerH - ((p.value - minV) / range) * innerH;
      return { x, y, ...p };
    });

    const linePath = points.map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x} ${p.y}`).join(' ');
    const areaPath = `${linePath} L ${points[points.length - 1].x} ${height - padY} L ${points[0].x} ${height - padY} Z`;
    const peak = stats.peakIndex >= 0 ? points[stats.peakIndex] : null;

    return { points, linePath, areaPath, peak };
  });
</script>

<div class="rounded-xl bg-white/90 dark:bg-card/95 border border-violet-100 dark:border-violet-900/30 p-2.5 shadow-sm w-full">
  <div class="flex items-center justify-between gap-2 mb-2">
    <div>
      <p class="text-[10px] font-medium text-violet-600/80 dark:text-violet-400 uppercase tracking-wide">
        Impact trend
      </p>
      <p class="text-lg font-bold text-foreground leading-tight">{stats.latest}</p>
    </div>
    {#if chart?.peak}
      <div class="rounded-lg bg-violet-600 text-white px-2 py-1 text-right">
        <p class="text-[9px] opacity-90">Peak</p>
        <p class="text-xs font-bold leading-none">{Math.round(chart.peak.value)}</p>
      </div>
    {/if}
  </div>

  {#if chart}
    <svg
      viewBox="0 0 {width} {height}"
      class="w-full h-auto max-h-20"
      preserveAspectRatio="xMidYMid meet"
      role="img"
      aria-label="Impact score trend over recent games"
    >
      <defs>
        <linearGradient id="mhImpactFill" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stop-color="rgb(124, 58, 237)" stop-opacity="0.35" />
          <stop offset="100%" stop-color="rgb(124, 58, 237)" stop-opacity="0.02" />
        </linearGradient>
      </defs>
      <path d={chart.areaPath} fill="url(#mhImpactFill)" />
      <path
        d={chart.linePath}
        fill="none"
        stroke="rgb(124, 58, 237)"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      {#each chart.points as point, i}
        <circle
          cx={point.x}
          cy={point.y}
          r={i === stats.peakIndex ? 3.5 : 2.5}
          fill={point.win ? 'rgb(34, 197, 94)' : 'rgb(239, 68, 68)'}
          stroke="white"
          stroke-width="1"
        />
      {/each}
    </svg>

    <div class="grid grid-cols-4 gap-1 mt-2 pt-2 border-t border-violet-100 dark:border-violet-900/30">
      {#each [
        { label: 'Avg', value: stats.avg },
        { label: 'Min', value: stats.min },
        { label: 'Max', value: stats.max },
        { label: 'Last', value: stats.latest },
      ] as col}
        <div class="text-center">
          <p class="text-[9px] text-muted-foreground">{col.label}</p>
          <p class="text-xs font-bold text-violet-600 dark:text-violet-400">{col.value}</p>
        </div>
      {/each}
    </div>
  {:else}
    <div class="h-14 flex items-center justify-center text-xs text-muted-foreground rounded-lg bg-violet-50/50 dark:bg-violet-950/20">
      Need more games
    </div>
  {/if}
</div>
