<script lang="ts">
  import { Card, CardContent, CardHeader } from '$lib/components/ui/card';
  import { cn } from '$lib/utils';

  export let title: string;
  export let className: string = '';
  export let children: any;
</script>

<Card class={cn('', className)}>
  <CardHeader>
    <h2 class="text-lg font-semibold text-foreground">{title}</h2>
  </CardHeader>
  <CardContent>
    {@render children?.()}
  </CardContent>
</Card>
