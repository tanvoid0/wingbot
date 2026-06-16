<script lang="ts">
  import type { Component } from 'svelte';
  import { cn } from '$lib/utils';

  interface Props {
    label: string;
    value: string | number;
    icon: Component;
    iconBg?: string;
    iconColor?: string;
    valueClass?: string;
    subtitle?: string;
    progress?: number;
    accent?: 'violet' | 'emerald' | 'amber' | 'rose' | 'sky' | 'orange';
  }

  let {
    label,
    value,
    icon: Icon,
    iconBg,
    iconColor,
    valueClass = '',
    subtitle,
    progress,
    accent = 'violet',
  }: Props = $props();

  const accentMap = {
    violet: { bg: 'bg-violet-500/10', icon: 'text-violet-600', bar: 'from-violet-500 to-purple-600' },
    emerald: { bg: 'bg-emerald-500/10', icon: 'text-emerald-600', bar: 'from-emerald-500 to-teal-500' },
    amber: { bg: 'bg-amber-500/10', icon: 'text-amber-600', bar: 'from-amber-500 to-orange-500' },
    rose: { bg: 'bg-rose-500/10', icon: 'text-rose-600', bar: 'from-rose-500 to-red-500' },
    sky: { bg: 'bg-sky-500/10', icon: 'text-sky-600', bar: 'from-sky-500 to-blue-500' },
    orange: { bg: 'bg-orange-500/10', icon: 'text-orange-600', bar: 'from-orange-500 to-amber-500' },
  };

  const theme = $derived(accentMap[accent]);
</script>

<div
  class="rounded-xl bg-white/90 dark:bg-card/95 border border-violet-100/80 dark:border-violet-900/25 p-2.5 shadow-sm"
>
  <div class="flex items-center justify-between gap-2">
    <div class="min-w-0 flex-1">
      <p class="text-[10px] font-medium text-muted-foreground uppercase tracking-wide">{label}</p>
      <p class={cn('text-lg font-bold leading-tight truncate', valueClass)}>{value}</p>
      {#if subtitle}
        <p class="text-[10px] text-muted-foreground mt-0.5 truncate">{subtitle}</p>
      {/if}
    </div>
    <div class={cn('p-1.5 rounded-lg shrink-0', iconBg ?? theme.bg)}>
      <Icon class={cn('w-3.5 h-3.5', iconColor ?? theme.icon)} />
    </div>
  </div>
  {#if progress != null}
    <div class="mt-2 h-1 rounded-full bg-violet-100 dark:bg-violet-950/40 overflow-hidden">
      <div
        class={cn('h-full rounded-full bg-gradient-to-r', theme.bar)}
        style="width: {Math.min(100, progress)}%"
      ></div>
    </div>
  {/if}
</div>
