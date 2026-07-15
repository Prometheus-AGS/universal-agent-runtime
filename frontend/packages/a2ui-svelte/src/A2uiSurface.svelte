<script lang="ts">
  import type { ComponentApi, SurfaceModel } from "@prometheus-ags/a2ui-core/v0_9";
  import { renderSemanticSurface, type SemanticRendererHandle } from "@prometheus-ags/a2ui-core/semantic-dom";

  let { surface }: { surface: SurfaceModel<ComponentApi> } = $props();
  let container: HTMLElement;
  let handle: SemanticRendererHandle | undefined;

  $effect(() => {
    if (!container || !surface) return;
    handle?.dispose();
    handle = renderSemanticSurface(surface, container);
    return () => handle?.dispose();
  });
</script>

<div bind:this={container} data-a2ui-svelte-surface aria-label="A2UI surface"></div>
