<script lang="ts">
  import { Command } from '@tauri-apps/api/shell'
  import { invoke } from "@tauri-apps/api/tauri"
  import Gallery from './Gallery.svelte';
  import { Store } from "tauri-plugin-store-api";
  import { message } from '@tauri-apps/api/dialog';

  let query = ""
  let searchMsg = ""
  let indexMsg = "No elements indexed yet"
  let isSearching = false 
  let notFoundMessage = "No result found. Maybe we're still loading your images, wait a bit!"
  let sliceImages = 25
  let isFirst
  let ms = 600000 // 10 minutes
  let clear
  let isIndexing = false

  async function isFirstTime(){
    const store = new Store(".settings.dat");
    if (await store.get("firstTime") === null) {
      await message('Welcome to the app! This is your first time here, so the app will index all your images. This will take some time, so please be patient. You can close the app and open it again when you think it\'s done. Enjoy!', 
      { title: 'ByteDetective Onboarding', okLabel: 'Got it!' });
      isFirst = true;
    } else{
      isFirst = false;
    }
    await store.set("firstTime", { value: false });
    await store.save();
  }

  async function search(){
    sliceImages = 25;
    if (query.trim().length === 0) {
      searchMsg = "The input box is empty 🙁";
      return;
  }
    isSearching = true;
    query = query.trim().toLocaleLowerCase();
    searchMsg = await invoke("search", { query });

    if (searchMsg.length === 0) {
      searchMsg = notFoundMessage;
    }

    isSearching = false;
  }

async function index() {
  isIndexing = true;
  const command = Command.sidecar('bin/python/test');
  indexMsg = "Syncing with your files... Don't bother, start searching!";

  try {
    const { stdout, stderr } = await command.execute();
    if (stdout) {
      indexMsg = stdout + " images indexed";
    }
  } catch (error) {
     indexMsg = "Something wrong happened, please close and retry."
  }
  isIndexing = false;
}

async function manual_index() {
  isIndexing = true;
  const command = Command.sidecar('bin/python/test');
  indexMsg = "Syncing with your files, please wait...";

  try {
    const { stdout, stderr } = await command.execute();
    if (stdout) {
      indexMsg = stdout + " images indexed";
    }
    console.log(stdout, stderr)
  } catch (error) {
     indexMsg = "Something wrong happened, please close and retry."
  }
  isIndexing = false;
}

function handleKeyDown(event) {
    if (event.key === "Enter" && !isSearching) {
        search();
    }
}


$: {
	 clearInterval(clear)
   if (!isIndexing) {
    clear = setInterval(index, ms);
  }
 }

(async () => {
  await isFirstTime();
  await index(); // Call the index() function when the app opens
})();

</script>

<div>
  <div class="row">
    <div class="search-bar-container">
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <span class="material-symbols-outlined" on:click={search}>search</span>
      <input class="search-bar" placeholder="A picture with..." bind:value={query} on:keydown={handleKeyDown}/>
    </div>
    <button class="index-button" on:click={manual_index}><img src="/index.png" class="index-image" alt="Index" /></button>
</div>

{#if !isFirst || indexMsg !== "Just checking for new images... Don't bother, start searching!"}
  {#key indexMsg}
    <p class="index">{indexMsg}</p>
  {/key}
{:else}
  <p class="index">Indexing... Please wait, that can take some time (~2s/image)</p>
{/if}

  {#if searchMsg === "The input box is empty 🙁"}
    <p>{searchMsg}</p>
  {:else if isSearching}
    <p>Searching...</p>
  {:else if searchMsg.length > 1 && !isSearching}
  <center>
    <div class="result-container">
    <Gallery gap={15} maxColumnWidth={200} loading="lazy">
        {#each searchMsg.slice(0, sliceImages) as msg}
           <!-- svelte-ignore a11y-missing-attribute -->
            <!-- svelte-ignore a11y-click-events-have-key-events -->
              <img src={"http://127.0.0.1:8080/6a4e786120cb00c1a0f85dc5528f75debff6eec8"+msg} class="image" alt={msg}/>
        {/each}
    </Gallery>
    </div>
    </center>
{/if}

{#if sliceImages < searchMsg.length}
 <button
		on:click={() => sliceImages = sliceImages + 25}
    id="loadmore"
    type="button">
    Load more
  </button>
{/if}

</div>
