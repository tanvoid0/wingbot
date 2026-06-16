<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';
  import { getRoleIcon } from '$lib/services/imageService';
  import { getRoleLabel, getImpactBadgeClass } from '$lib/utils/matchHistory';

  interface Props {
    roleStats: Record<string, unknown>;
  }

  let { roleStats }: Props = $props();

  const entries = $derived(Object.entries(roleStats || {}));
</script>

{#if entries.length > 0}
  <div class="rounded-xl bg-white/80 dark:bg-card/90 border border-violet-100 dark:border-violet-900/30 p-2.5 space-y-2 max-w-md">
    <h4 class="text-xs font-semibold text-muted-foreground">Roles</h4>
    <div class="space-y-1.5">
      {#each entries as [role, data]}
        {@const d = data as { games?: number; win_rate?: number; avg_impact?: number }}
        <div class="rounded-lg bg-violet-50/50 dark:bg-violet-950/20 px-2.5 py-2 border border-violet-100/60 dark:border-violet-900/20">
          <div class="flex items-center gap-2">
            <span class="text-sm" title={getRoleLabel(role)}>{getRoleIcon(role)}</span>
            <div class="flex-1 min-w-0">
              <p class="text-xs font-medium">{getRoleLabel(role)}</p>
              <p class="text-[10px] text-muted-foreground">{d.games ?? 0}g · {Math.round(d.win_rate || 0)}% WR</p>
            </div>
            <Badge variant="outline" class="text-[9px] h-5 {getImpactBadgeClass(d.avg_impact || 0)}">
              {Math.round(d.avg_impact || 0)}
            </Badge>
          </div>
          <div class="mt-1.5 h-1 rounded-full bg-violet-100 dark:bg-violet-950/50 overflow-hidden">
            <div
              class="h-full rounded-full bg-gradient-to-r from-violet-500 to-purple-500"
              style="width: {Math.min(100, d.win_rate || 0)}%"
            ></div>
          </div>
        </div>
      {/each}
    </div>
  </div>
{/if}
