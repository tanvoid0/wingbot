<script lang="ts">
  import { AlertCircle, CheckCircle, Clock, Lightbulb, Loader2, RefreshCw, Sparkles } from '@lucide/svelte';
  import { cn } from '$lib/utils';

  interface Insight {
    type: string;
    title: string;
    message: string;
    source?: string;
  }

  interface Props {
    insights: Insight[];
    aiLoading?: boolean;
    aiError?: string | null;
    onRetryAi?: () => void;
  }

  let { insights, aiLoading = false, aiError = null, onRetryAi }: Props = $props();
  let showInsights = $state(true);

  function insightStyles(type: string) {
    switch (type) {
      case 'positive':
      case 'success':
        return {
          icon: CheckCircle,
          card: 'bg-emerald-50/80 dark:bg-emerald-950/20 border-emerald-200/60',
          title: 'text-emerald-700 dark:text-emerald-400',
          iconColor: 'text-emerald-600',
        };
      case 'negative':
      case 'error':
        return {
          icon: AlertCircle,
          card: 'bg-rose-50/80 dark:bg-rose-950/20 border-rose-200/60',
          title: 'text-rose-700 dark:text-rose-400',
          iconColor: 'text-rose-600',
        };
      case 'warning':
        return {
          icon: AlertCircle,
          card: 'bg-amber-50/80 dark:bg-amber-950/20 border-amber-200/60',
          title: 'text-amber-700 dark:text-amber-400',
          iconColor: 'text-amber-600',
        };
      case 'info':
        return {
          icon: Clock,
          card: 'bg-blue-50/80 dark:bg-blue-950/20 border-blue-200/60',
          title: 'text-blue-700 dark:text-blue-400',
          iconColor: 'text-blue-600',
        };
      default:
        return {
          icon: Clock,
          card: 'bg-violet-50/80 dark:bg-violet-950/20 border-violet-200/60',
          title: 'text-foreground',
          iconColor: 'text-violet-600',
        };
    }
  }
</script>

{#if insights.length > 0 || aiLoading || aiError}
  <div class="rounded-xl bg-white/80 dark:bg-card/90 border border-violet-100 dark:border-violet-900/30 p-2.5 space-y-2 max-w-md">
    <div class="flex items-center justify-between">
      <h4 class="text-xs font-semibold flex items-center gap-1.5 text-muted-foreground">
        <Lightbulb class="w-3.5 h-3.5 text-amber-500" />
        Insights
        {#if aiLoading}
          <Loader2 class="w-3 h-3 animate-spin text-violet-500" />
        {/if}
      </h4>
      <button
        type="button"
        onclick={() => (showInsights = !showInsights)}
        class="text-[10px] text-violet-600 hover:text-violet-700"
      >
        {showInsights ? 'Hide' : 'Show'}
      </button>
    </div>
    {#if showInsights}
      <div class="space-y-1.5">
        {#each insights as insight}
          {@const styles = insightStyles(insight.type)}
          {@const Icon = styles.icon}
          <div class={cn('rounded-lg border p-2 flex gap-2', styles.card)}>
            <Icon class={cn('w-3.5 h-3.5 mt-0.5 shrink-0', styles.iconColor)} />
            <div class="min-w-0">
              <div class="flex items-center gap-1.5">
                <p class={cn('text-xs font-semibold', styles.title)}>{insight.title}</p>
                {#if insight.source === 'ai'}
                  <Sparkles class="w-3 h-3 text-violet-500 shrink-0" title="AI insight" />
                {/if}
              </div>
              <p class="text-[10px] text-muted-foreground mt-0.5 leading-snug">{insight.message}</p>
            </div>
          </div>
        {/each}

        {#if aiError && !aiLoading}
          <div class="rounded-lg border border-amber-200/60 bg-amber-50/80 dark:bg-amber-950/20 dark:border-amber-900/40 p-2 flex gap-2">
            <AlertCircle class="w-3.5 h-3.5 mt-0.5 shrink-0 text-amber-600" />
            <div class="min-w-0 flex-1">
              <p class="text-[10px] text-amber-800 dark:text-amber-300 leading-snug">{aiError}</p>
              {#if onRetryAi}
                <button
                  type="button"
                  onclick={onRetryAi}
                  class="mt-1 inline-flex items-center gap-1 text-[10px] font-medium text-violet-600 hover:text-violet-700"
                >
                  <RefreshCw class="w-3 h-3" />
                  Retry AI insights
                </button>
              {/if}
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
{/if}
