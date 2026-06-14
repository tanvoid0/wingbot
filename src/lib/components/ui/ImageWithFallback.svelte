<script lang="ts">
  import { onMount } from 'svelte';

  interface Props {
    src: string | null;
    alt: string;
    fallback: string;
    class?: string;
    width?: number;
    height?: number;
    loading?: 'lazy' | 'eager';
  }

  let {
    src,
    alt,
    fallback,
    class: className = '',
    width,
    height,
    loading = 'lazy'
  }: Props = $props();

  let imageLoaded = $state(false);
  let imageError = $state(false);
  let imgElement = $state<HTMLImageElement>();

  // Reset states when src changes
  $effect(() => {
    if (src) {
      imageLoaded = false;
      imageError = false;
    }
  });

  function handleImageLoad() {
    imageLoaded = true;
    imageError = false;
  }

  function handleImageError() {
    imageLoaded = false;
    imageError = true;
  }

  // Preload image to check if it's valid
  async function preloadImage() {
    if (!src) {
      imageError = true;
      return;
    }

    try {
      const img = new Image();
      img.onload = () => {
        imageLoaded = true;
        imageError = false;
      };
      img.onerror = () => {
        imageLoaded = false;
        imageError = true;
      };
      img.src = src;
    } catch {
      imageError = true;
    }
  }

  onMount(() => {
    if (src) {
      preloadImage();
    } else {
      imageError = true;
    }
  });
</script>

<div class="relative {className}" style="width: {width}px; height: {height}px;">
  {#if src && !imageError}
    <img
      bind:this={imgElement}
      src={src}
      alt={alt}
      class="w-full h-full object-cover rounded"
      loading={loading}
      onload={handleImageLoad}
      onerror={handleImageError}
      style="opacity: {imageLoaded ? 1 : 0}; transition: opacity 0.2s ease-in-out;"
    />
  {/if}
  
  {#if imageError || !src}
    <div class="w-full h-full bg-gradient-to-br from-muted to-muted/50 rounded flex items-center justify-center text-muted-foreground font-bold">
      {fallback}
    </div>
  {/if}
  
  {#if src && !imageLoaded && !imageError}
    <div class="absolute inset-0 bg-muted/50 rounded flex items-center justify-center">
      <div class="w-4 h-4 border-2 border-primary border-t-transparent rounded-full animate-spin"></div>
    </div>
  {/if}
</div>
