<script lang="ts">
  import { Card, CardContent, CardHeader } from '$lib/components/ui/card';
  import { cn } from '$lib/utils';

  export let title: string;
  export let className: string = '';
  export let children: any;
</script>

<Card class={cn('', className)}>
  <CardHeader class="py-3 px-4">
    <h2 class="text-sm font-semibold text-foreground">{title}</h2>
  </CardHeader>
  <CardContent class="pt-0 px-4 pb-4">
    {@render children?.()}
  </CardContent>
</Card>
