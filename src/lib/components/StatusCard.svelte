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
  <CardHeader class="pb-3">
    <div class="flex items-center justify-between">
      <div>
        <p class="text-sm font-medium text-muted-foreground">{title}</p>
        <p class="text-2xl font-semibold text-foreground">{value}</p>
      </div>
      <div class={cn('p-2 rounded-full', config.bgColor)}>
        <svelte:component this={icon} class={cn('w-6 h-6', config.iconColor)} />
      </div>
    </div>
  </CardHeader>
  
  {#if actionLabel && onAction}
    <CardContent class="pt-0">
      <Button 
        variant="outline" 
        size="sm" 
        class="w-full"
        onclick={onAction}
      >
        {actionLabel}
      </Button>
    </CardContent>
  {/if}
</Card>
