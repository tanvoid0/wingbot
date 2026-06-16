<script lang="ts">
  import { Card, CardContent, CardHeader } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { cn } from '$lib/utils';
  import type { SvelteComponent } from 'svelte';

  export let title: string;
  export let value: string;
  export let icon: any;
  export let status: 'success' | 'error' | 'warning' | 'neutral' = 'neutral';
  export let actionLabel: string | undefined = undefined;
  export let onAction: (() => void) | undefined = undefined;
  export let className: string = '';

  const statusConfig = {
    success: {
      bgColor: 'bg-green-100 dark:bg-green-900/20',
      iconColor: 'text-green-600 dark:text-green-400'
    },
    error: {
      bgColor: 'bg-red-100 dark:bg-red-900/20',
      iconColor: 'text-red-600 dark:text-red-400'
    },
    warning: {
      bgColor: 'bg-yellow-100 dark:bg-yellow-900/20',
      iconColor: 'text-yellow-600 dark:text-yellow-400'
    },
    neutral: {
      bgColor: 'bg-gray-100 dark:bg-gray-700',
      iconColor: 'text-gray-600 dark:text-gray-400'
    }
  };

  const config = statusConfig[status];
</script>

<Card class={cn('', className)}>
  <CardHeader class="py-3 px-4">
    <div class="flex items-center justify-between gap-2">
      <div class="min-w-0">
        <p class="text-xs font-medium text-muted-foreground">{title}</p>
        <p class="text-lg font-semibold text-foreground truncate">{value}</p>
      </div>
      <div class={cn('p-1.5 rounded-full shrink-0', config.bgColor)}>
        <svelte:component this={icon} class={cn('w-4 h-4', config.iconColor)} />
      </div>
    </div>
  </CardHeader>

  {#if actionLabel && onAction}
    <CardContent class="pt-0 pb-3 px-4">
      <Button
        variant="outline"
        size="sm"
        class="w-full h-8"
        onclick={onAction}
      >
        {actionLabel}
      </Button>
    </CardContent>
  {/if}
</Card>
