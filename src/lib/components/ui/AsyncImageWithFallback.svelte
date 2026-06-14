<script lang="ts">
  import { onMount } from 'svelte';
  import ImageWithFallback from './ImageWithFallback.svelte';
  
  interface Props {
    getImageUrl: () => Promise<string | null>;
    alt: string;
    fallback: string;
    class?: string;
    width?: number;
    height?: number;
  }
  
  let { getImageUrl, alt, fallback, class: className = '', width, height }: Props = $props();
  
  let imageUrl = $state<string | null>(null);
  let isLoading = $state(true);
  let error = $state(false);
  
  onMount(async () => {
    try {
      isLoading = true;
      error = false;
      imageUrl = await getImageUrl();
    } catch (err) {
      console.warn('Failed to load image URL:', err);
      error = true;
      imageUrl = null;
    } finally {
      isLoading = false;
    }
  });
</script>

<ImageWithFallback
  src={imageUrl}
  alt={alt}
  fallback={fallback}
  class={className}
  width={width}
  height={height}
/>
