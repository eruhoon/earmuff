<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Menu, MenuItem, PredefinedMenuItem } from "@tauri-apps/api/menu";

  // Configuration Constants (Eliminating magic values)
  const DEFAULT_PROCESS = "LOSTARK.exe";
  const FALLBACK_PROCESSES = ["LOSTARK.exe", "chrome.exe", "discord.exe", "spotify.exe"];
  const STATUS_POLL_INTERVAL_MS = 1000;
  const LIST_POLL_INTERVAL_MS = 4000;
  const PROCESS_STORAGE_KEY = "target_process";

  let targetProcess = $state(DEFAULT_PROCESS);
  let isRunning = $state(false);
  let isMuted = $state(false);
  
  let processList = $state<string[]>([]);
  let iconCache = $state<Record<string, string>>({});

  // Poll process status and mute status
  async function checkStatus() {
    try {
      const running = await invoke<boolean>("is_process_running", { processName: targetProcess });
      isRunning = running;
      if (running) {
        isMuted = await invoke<boolean>("get_mute_status", { processName: targetProcess });
      }
    } catch (e) {
      console.error("Status check failed:", e);
    }
  }

  // Toggle mute
  async function handleToggle() {
    if (!isRunning) return;
    try {
      isMuted = await invoke<boolean>("toggle_mute", { processName: targetProcess });
    } catch (e) {
      console.error("Toggle failed:", e);
    }
  }

  // Load icon for a process
  async function loadIcon(name: string) {
    if (iconCache[name] !== undefined) return;
    try {
      const dataUrl = await invoke<string>("get_process_icon", { processName: name });
      iconCache[name] = dataUrl;
    } catch (e) {
      console.warn(`Failed to load native icon for ${name}:`, e);
      iconCache[name] = ""; // Mark as failed
    }
  }

  // Pre-load icons for a list of processes
  function loadListIcons(list: string[]) {
    list.forEach(name => loadIcon(name));
  }

  // Update list of audio processes running on the system
  async function updateProcessList() {
    try {
      const list = await invoke<string[]>("get_audio_processes");
      let cleanedList = list.filter(p => p.toLowerCase() !== "idle" && p.toLowerCase() !== "system");
      if (!cleanedList.includes(targetProcess)) {
        cleanedList = [targetProcess, ...cleanedList];
      }
      processList = cleanedList;
      loadListIcons(processList); // Pre-cache icons in the background
    } catch (err) {
      console.error("Failed to fetch audio processes:", err);
      if (processList.length === 0) {
        processList = FALLBACK_PROCESSES;
        loadListIcons(processList);
      }
    }
  }

  // Handle scroll wheel to directly change the target process (no click/contextmenu needed!)
  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    if (processList.length <= 1) return;
    
    let idx = processList.indexOf(targetProcess);
    if (idx === -1) idx = 0;
    
    if (e.deltaY > 0) {
      // Scroll down -> next process
      idx = (idx + 1) % processList.length;
    } else if (e.deltaY < 0) {
      // Scroll up -> previous process
      idx = (idx - 1 + processList.length) % processList.length;
    }
    
    targetProcess = processList[idx];
    localStorage.setItem(PROCESS_STORAGE_KEY, targetProcess);
    checkStatus(); // Instantly update status for the new target
  }

  // Handle right-click on the drag handle to popup a custom context menu
  async function handleDragHandleRightClick(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    try {
      const unmuteAllItem = await MenuItem.new({
        id: "unmute_all",
        text: "전체 음소거 해제",
        action: async () => {
          await invoke("unmute_all_processes");
          checkStatus(); // Instantly update status for the widget
        }
      });
      
      const separator = await PredefinedMenuItem.new({
        item: "Separator"
      });

      const quitItem = await MenuItem.new({
        id: "quit",
        text: "종료",
        action: async () => {
          await invoke("exit_app");
        }
      });

      const menu = await Menu.new({
        items: [unmuteAllItem, separator, quitItem]
      });
      await menu.popup();
    } catch (err) {
      console.error("Failed to popup context menu:", err);
    }
  }

  // Helper to remove .exe for cleaner display
  function getDisplayName(filename: string) {
    if (filename.toLowerCase().endsWith(".exe")) {
      return filename.slice(0, -4);
    }
    return filename;
  }

  // Auto-load target process icon on change
  $effect(() => {
    loadIcon(targetProcess);
  });

  onMount(() => {
    const saved = localStorage.getItem(PROCESS_STORAGE_KEY);
    if (saved) {
      targetProcess = saved;
    }
    
    checkStatus();
    updateProcessList();
    
    // Poll target process status every second
    const statusInterval = setInterval(checkStatus, STATUS_POLL_INTERVAL_MS);
    // Poll system audio process list every 4 seconds
    const listInterval = setInterval(updateProcessList, LIST_POLL_INTERVAL_MS);
    
    return () => {
      clearInterval(statusInterval);
      clearInterval(listInterval);
    };
  });
</script>

{#snippet processIcon(name: string)}
  {#if iconCache[name]}
    <!-- Render the extracted native Windows icon -->
    <img src={iconCache[name]} alt={name} class="process-logo native-icon" />
  {:else}
    <!-- Fallback beautiful SVGs if process not running or icon extraction failed -->
    {#if name.toLowerCase().includes("lostark")}
      <!-- Lost Ark Stylized Golden Winged Sword Crest -->
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" class="process-logo lostark-logo">
        <path d="M12 3v13M10 13h4M12 16v3M11 19h2" stroke="#ffd700" stroke-width="2.2" stroke-linecap="round"></path>
        <path d="M5 6c2 1 4 4 7 10" stroke="#ffd700" stroke-width="1.8" stroke-linecap="round"></path>
        <path d="M19 6c-2 1-4 4-7 10" stroke="#ffd700" stroke-width="1.8" stroke-linecap="round"></path>
        <path d="M12 8l1.5 1.5-1.5 1.5-1.5-1.5z" fill="#ffd700"></path>
      </svg>
    {:else if name.toLowerCase().includes("chrome") || name.toLowerCase().includes("brave") || name.toLowerCase().includes("edge") || name.toLowerCase().includes("browser") || name.toLowerCase().includes("whale")}
      <!-- Browser Globe Icon -->
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" class="process-logo browser-logo">
        <circle cx="12" cy="12" r="10"></circle>
        <line x1="2" y1="12" x2="22" y2="12"></line>
        <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
      </svg>
    {:else if name.toLowerCase().includes("discord")}
      <!-- Discord Chat Icon -->
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" class="process-logo discord-logo">
        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
        <circle cx="9" cy="10" r="1" fill="currentColor"></circle>
        <circle cx="15" cy="10" r="1" fill="currentColor"></circle>
      </svg>
    {:else if name.toLowerCase().includes("spotify") || name.toLowerCase().includes("music")}
      <!-- Music Note Icon -->
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" class="process-logo music-logo">
        <path d="M9 18V5l12-2v13"></path>
        <circle cx="6" cy="18" r="3"></circle>
        <circle cx="18" cy="16" r="3"></circle>
      </svg>
    {:else}
      <!-- Generic App Icon -->
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" class="process-logo default-logo">
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
        <line x1="9" y1="3" x2="9" y2="21"></line>
      </svg>
    {/if}
  {/if}
{/snippet}

<main class="widget-container" data-tauri-drag-region>
  <div class="capsule">
    <!-- 1. Drag Handle -->
    <div 
      class="drag-handle" 
      data-tauri-drag-region 
      oncontextmenu={handleDragHandleRightClick}
      title="드래그하여 이동 / 우클릭: 종료"
    >
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" class="grip-icon" data-tauri-drag-region>
        <circle cx="8" cy="6" r="1.5" fill="currentColor" data-tauri-drag-region></circle>
        <circle cx="16" cy="6" r="1.5" fill="currentColor" data-tauri-drag-region></circle>
        <circle cx="8" cy="12" r="1.5" fill="currentColor" data-tauri-drag-region></circle>
        <circle cx="16" cy="12" r="1.5" fill="currentColor" data-tauri-drag-region></circle>
        <circle cx="8" cy="18" r="1.5" fill="currentColor" data-tauri-drag-region></circle>
        <circle cx="16" cy="18" r="1.5" fill="currentColor" data-tauri-drag-region></circle>
      </svg>
    </div>

    <!-- Separator Line -->
    <div class="separator"></div>

    <!-- 2. Integrated Action Button (Click to toggle mute, Scroll to switch target) -->
    <div 
      class="action-button" 
      class:running={isRunning} 
      class:muted={isMuted} 
      onclick={handleToggle}
      onwheel={handleWheel}
      title={`좌클릭: 음소거 토글 / 휠 스크롤: 대상 프로그램 변경 (현재: ${targetProcess})`}
      role="button"
      tabindex="0"
      onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleToggle(); } }}
    >
      <div class="icon-container" class:mute-slash={isMuted && isRunning}>
        {@render processIcon(targetProcess)}
      </div>
      
      <span class="label">
        {#if !isRunning}
          {getDisplayName(targetProcess)} OFF
        {:else if isMuted}
          MUTED
        {:else}
          SOUND
        {/if}
      </span>
    </div>
  </div>
</main>

<style>
  :global(html) {
    background: transparent !important;
  }
  
  :global(body) {
    background: transparent !important;
    margin: 0;
    padding: 0;
    overflow: hidden;
    user-select: none;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  }

  .widget-container {
    width: 160px;
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
  }

  .capsule {
    display: flex;
    align-items: center;
    width: 130px;
    height: 36px;
    border-radius: 18px;
    border: 1.5px solid rgba(255, 255, 255, 0.08);
    background: rgba(18, 18, 18, 0.75);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    box-shadow: 0 4px 10px rgba(0, 0, 0, 0.4);
    box-sizing: border-box;
    overflow: hidden;
    transition: all 0.2s ease;
  }

  /* Drag Handle Area */
  .drag-handle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 100%;
    color: rgba(255, 255, 255, 0.3);
    cursor: grab;
    transition: color 0.2s ease;
  }

  .drag-handle:hover {
    color: rgba(255, 255, 255, 0.65);
  }

  .drag-handle:active {
    cursor: grabbing;
    color: rgba(255, 255, 255, 0.95);
  }

  .grip-icon {
    width: 12px;
    height: 12px;
  }

  /* Separator */
  .separator {
    width: 1px;
    height: 16px;
    background: rgba(255, 255, 255, 0.12);
  }

  /* Integrated Action Button (Icon + Label unified) */
  .action-button {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    width: 104px;
    height: 100%;
    color: #888888;
    font-size: 10px;
    font-weight: 800;
    letter-spacing: 0.5px;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    box-sizing: border-box;
    user-select: none;
    outline: none;
  }

  .action-button:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .action-button:active {
    transform: scale(0.96);
  }

  .action-button.running {
    color: #4ade80;
  }

  .action-button.running:hover {
    background: rgba(74, 222, 128, 0.08);
  }

  .action-button.muted {
    color: #f87171;
  }

  .action-button.muted:hover {
    background: rgba(248, 113, 113, 0.08);
  }

  .icon-container {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    position: relative;
  }

  /* Slash line over icon when muted */
  .icon-container.mute-slash::after {
    content: '';
    position: absolute;
    width: 18px;
    height: 2px;
    background: #f87171;
    transform: rotate(-45deg);
    box-shadow: 0 0 2px rgba(0,0,0,0.6);
  }

  .label {
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 68px;
  }

  /* SVGs styles */
  :global(.process-logo) {
    width: 100%;
    height: 100%;
    flex-shrink: 0;
    object-fit: contain;
  }
  
  :global(.native-icon) {
    width: 14px;
    height: 14px;
    filter: drop-shadow(0 1px 3px rgba(0, 0, 0, 0.3));
  }

  :global(.lostark-logo) {
    filter: drop-shadow(0 0 2.5px rgba(255, 215, 0, 0.4));
  }

  :global(.browser-logo) {
    color: #60a5fa;
  }

  :global(.discord-logo) {
    color: #818cf8;
  }

  :global(.music-logo) {
    color: #34d399;
  }

  :global(.default-logo) {
    color: #a3a3a3;
  }
</style>
