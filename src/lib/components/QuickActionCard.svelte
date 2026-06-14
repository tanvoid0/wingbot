<script lang="ts">
  import { Card, CardContent } from '$lib/components/ui/card';
  import { cn } from '$lib/utils';
  import type { SvelteComponent } from 'svelte';

  export let title: string;
  export let description: string;
  export let icon: any;
  export let href: string;
  export let variant: 'blue' | 'green' | 'purple' = 'blue';
  export let className: string = '';

  const variantConfig = {
    blue: {
      bgColor: 'bg-blue-50 dark:bg-blue-900/20',
      borderColor: 'border-blue-200 dark:border-blue-800',
      hoverBgColor: 'hover:bg-blue-100 dark:hover:bg-blue-900/40',
      titleColor: 'text-blue-900 dark:text-blue-100',
      descriptionColor: 'text-blue-700 dark:text-blue-300',
      iconColor: 'text-blue-600 dark:text-blue-400'
    },
    green: {
      bgColor: 'bg-green-50 dark:bg-green-900/20',
      borderColor: 'border-green-200 dark:border-green-800',
      hoverBgColor: 'hover:bg-green-100 dark:hover:bg-green-900/40',
      titleColor: 'text-green-900 dark:text-green-100',
      descriptionColor: 'text-green-700 dark:text-green-300',
      iconColor: 'text-green-600 dark:text-green-400'
    },
    purple: {
      bgColor: 'bg-purple-50 dark:bg-purple-900/20',
      borderColor: 'border-purple-200 dark:border-purple-800',
      hoverBgColor: 'hover:bg-purple-100 dark:hover:bg-purple-900/40',
      titleColor: 'text-purple-900 dark:text-purple-100',
      descriptionColor: 'text-purple-700 dark:text-purple-300',
      iconColor: 'text-purple-600 dark:text-purple-400'
    }
  };

  const config = variantConfig[variant];
</script>

<a href={href} class="block">
  <Card class={cn(
    'border transition-colors cursor-pointer',
    config.bgColor,
    config.borderColor,
    config.hoverBgColor,
    className
  )}>
    <CardContent class="p-4">
      <div class="flex items-center">
        <svelte:component this={icon} class={cn('w-6 h-6 mr-3', config.iconColor)} />
        <div>
          <p class={cn('font-medium', config.titleColor)}>{title}</p>
          <p class={cn('text-sm', config.descriptionColor)}>{description}</p>
        </div>
      </div>
    </CardContent>
  </Card>
</a>
