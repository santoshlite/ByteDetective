<script>
// @ts-nocheck
    import { onMount } from "svelte";
    import { tick } from "svelte";
    import { invoke } from "@tauri-apps/api/tauri";

    export let gap = 10;
    export let maxColumnWidth = 250;
    export let hover = false;
    export let loading;

    let slotHolder = null;
    let columns = [];
    let galleryWidth = 0;
    let columnCount = 0;

    $: columnCount = parseInt(String(galleryWidth / maxColumnWidth)) || 1;
    $: columnCount && Draw();
    $: galleryStyle = `grid-template-columns: repeat(${columnCount}, 1fr); --gap: ${gap}px`;

    onMount(Draw);

    async function open_file(dir){
        console.log(dir);
        await invoke("open_file_macos", { dir } );
  }

    async function Draw() {
        await tick();

        if (!slotHolder) {
            return;
        }

        const images = Array.from(slotHolder.childNodes).filter(
            (child) => child.tagName === "IMG"
        );
        columns = [];

        // Fill the columns with image URLs
        for (let i = 0; i < images.length; i++) {
            const idx = i % columnCount;
            columns[idx] = [
                ...(columns[idx] || []),
                { src: images[i].src, alt: images[i].alt, class: images[i].className },
            ];
        }
    }
</script>

<div
    id="slotHolder"
    bind:this={slotHolder}
    on:DOMNodeInserted={Draw}
    on:DOMNodeRemoved={Draw}
>
    <slot />
</div>

{#if columns}
    <div id="gallery" bind:clientWidth={galleryWidth} style={galleryStyle}>
        {#each columns as column}
            <div class="column">
                {#each column as img}
                <div class="img-container">
                    <!-- svelte-ignore a11y-click-events-have-key-events -->
                    <img
                        src={img.src}
                        alt={img.alt}
                        on:click={()=>open_file(img.alt)}
                        class="{hover === true ? "img-hover" : ""} {img.class}"
                        loading={loading}
                    />
                    <div class="middle">
                        <span class="material-symbols-outlined">
                            open_in_new
                            </span>                        
                        </div>
                    </div>
                {/each}
            </div>
        {/each}
    </div>
{/if}

<style>
    #slotHolder {
        display: none;
    }
    #gallery {
        width: 100%;
        display: grid;
        gap: var(--gap);
    }
    #gallery .column {
        display: flex;
        flex-direction: column;
    }
    #gallery .column * {
        width: 100%;
        margin-top: var(--gap);
    }
    #gallery .column *:nth-child(1) {
        margin-top: 0;
    }
    .img-hover {
        opacity: 0.9;
        transition: all 0.2s;
    }
    .img-hover:hover {
        opacity: 1;
        transform: scale(1.05);
    }

    .middle {
        position: absolute;
        right: 20px;
        top: -10px;
        display: none;
        border-radius: 12px;
        width: 30px !important;
    }

    .img-container {
        position: relative;
    }
    
    .img-container:hover .middle {
        display: block;
    }

    .material-symbols-outlined {
    border: 0.5px solid #545768;
    color: #a2a4af;
    background: #2a2c37;
    border-radius: 5px;
    font-variation-settings:
    'FILL' 0,
    'wght' 400,
    'GRAD' 0,
    'opsz' 48
}
</style>
