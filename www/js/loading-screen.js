//@ts-check

/**
 * Retro Loading Screen Module
 * Displays a retro-style loading screen when navigating between pages
 */

// Configuration options for loading screen
const config = {
    // Enable or disable loading screen delay (false = minimal delay)
    enableDelay: false,
    // Minimum animation time in milliseconds (used when enableDelay is false)
    minAnimationTime: 300,
    // Maximum animation time in milliseconds (used when enableDelay is true)
    maxAnimationTime: 1500,
    // Array of ASCII art to display during loading
    loadingMessages: [
        // Coffee break
        `       ╭──────╮
       │      │
       │ ☕   │  Loading...
       │      │  Please wait
       ╰──────╯`,

        // Hamster wheel
        `      ┌─────────────┐
      │  ┌─┐       │
      │ (◕‿◕)  ⟳   │  Spinning up
      │  └─┘       │  the hamsters
      └─────────────┘`,

        // Rubber duck debugging
        `        ___
       (o o)_____/
        (_____)    Debugging
        | | |      with style
        | | |`,

        // Loading pizza
        `      ╭─────────╮
      │  🍕🍕🍕  │
      │  🍕💾🍕  │  Delivering
      │  🍕🍕🍕  │  hot data
      ╰─────────╯`,

        // Server thinking
        `      ┌─────────┐
      │ [▓▓▓▓▓] │
      │  │ │ │  │  Server is
      │  ┴ ┴ ┴  │  thinking...
      └─────────┘`,

        // Pixel cat
        `      ╱|、
     (˚ˎ 。7      Fetching
     |、˜〵       adorable
     じしˍ,)ノ    content`,

        // Retro computer
        `      ┌───────────┐
      │ ■ ■ ■ ■ □ │
      │           │  Computing
      │   ▓▓▓▓▓   │  really hard
      └───────────┘
         ▓▓▓▓▓▓▓`,

        // Progress arrows
        `      ╔═══════════╗
      ║ ═══►     ║
      ║    ═══►  ║  Making
      ║       ═══►  progress
      ╚═══════════╝`,

        // Loading burrito
        `      ╭──────╮
      │ ▓▓▓▓ │  Wrapping
      │ ▓🌯▓ │  up your
      │ ▓▓▓▓ │  request
      ╰──────╯`,

        // Disco ball
        `        ⚬ ⚬ ⚬
       ⚬  ◆  ⚬
      ⚬  ◆◆◆  ⚬   Loading
       ⚬  ◆  ⚬    disco mode
        ⚬ ⚬ ⚬`,

        // Rocket launch
        `         /\\
        /  \\
       | ▓▓ |     Launching
       | ▓▓ |     in 3...2...1
      /|▓▓▓▓|\\
       \\ ~~ /
        ~~~~`,

        // Sleepy terminal
        `      ┌─────────┐
      │  ─  ─  │
      │    ω   │  Wake up
      │  ╰───╯ │  terminal!
      └─────────┘`,

        // Dancing dots
        `      ●   ●   ●
       ●   ●   ●
      ●   ●   ●    Loading
       ●   ●   ●   beat by
      ●   ●   ●    beat`,

        // Wizard hat
        `         ★
        ╱ ╲
       ╱   ╲      Casting
      ╱  ⚡  ╲     loading
     ╱───────╲    spell
    ╱─────────╲`,

        // Gears turning
        `      ⚙    ⚙
       ⚙  ⚙  ⚙   Turning
      ⚙  ⚙  ⚙    the gears
       ⚙  ⚙  ⚙   of progress
        ⚙  ⚙`,

        // Pac-Man loading
        `      ◀ ● ● ●
         ᗧ···     Chomping
      ◀ ● ● ●     data`,

        // Matrix style
        `      ⎡ 1 0 1 ⎤
      ⎢ 0 1 0 ⎥   Following
      ⎣ 1 0 1 ⎦   the rabbit`,

        // Hourglass
        `       ╱╲
      │  │
       ╲╱      Time is
       ╱╲      relative
      │▓▓│
       ╲╱`,

        // Musical notes
        `      ♪ ♫ ♪
       ♫   ♫      Loading
      ♪     ♪     to the
       ♫   ♫      beat
      ♪ ♫ ♪`,

        // Ninja loading
        `      ┌─────┐
      │ ◉ ◉ │
      │  ▼  │    Stealth
      └─────┘    loading
       │   │     mode
       └   ┘`,
    ],
    // Use existing site theme
    useExistingTheme: true,
};

// DOM Elements
let loadingScreen;
let loadingBar;
let loadingTextContainer;
let isInitialized = false;

/**
 * Creates the loading screen DOM structure
 */
function createLoadingScreen() {
    // Ensure we have access to CSS variables by extracting them from the document
    const computedStyle = getComputedStyle(document.documentElement);

    // Create loading screen container
    loadingScreen = document.createElement("div");
    loadingScreen.className = "retro-loading-screen";

    // Create subtle effects
    const crtEffect = document.createElement("div");
    crtEffect.className = "retro-crt-effect";

    const scanLine = document.createElement("div");
    scanLine.className = "retro-scan-line";

    // Create loading content container
    const loadingContent = document.createElement("div");
    loadingContent.className = "retro-loading-content";

    // Create loading header
    const loadingHeader = document.createElement("div");
    loadingHeader.className = "retro-loading-header";
    loadingHeader.textContent = "Loading";

    // Create loading bar container and bar
    const loadingBarContainer = document.createElement("div");
    loadingBarContainer.className = "retro-loading-bar-container";

    loadingBar = document.createElement("div");
    loadingBar.className = "retro-loading-bar";

    loadingBarContainer.appendChild(loadingBar);

    // Create loading text container
    loadingTextContainer = document.createElement("div");
    loadingTextContainer.className = "retro-loading-text";

    // Create cursor
    const cursor = document.createElement("div");
    cursor.className = "retro-loading-cursor retro-blink";
    cursor.innerHTML = "...";

    // Assemble the loading screen
    loadingContent.appendChild(loadingHeader);
    loadingContent.appendChild(loadingBarContainer);
    loadingContent.appendChild(loadingTextContainer);
    loadingContent.appendChild(cursor);

    loadingScreen.appendChild(crtEffect);
    loadingScreen.appendChild(scanLine);
    loadingScreen.appendChild(loadingContent);

    // Add to document
    document.body.appendChild(loadingScreen);
    isInitialized = true;
}

/**
 * Shows a loading message with a typewriter effect
 * @param {string} message - The message to display
 * @param {number} index - The index of the message line
 * @returns {Promise<void>}
 */
function showLoadingMessage(message, index) {
    return new Promise((resolve) => {
        const lineElement = document.createElement("div");
        lineElement.className = "retro-loading-text-line";
        lineElement.style.textAlign = "center";
        lineElement.innerHTML = `<pre style="display: inline-block; text-align: left;">${message}</pre>`;

        loadingTextContainer.appendChild(lineElement);

        // Delay before showing (minimal or none when delays disabled)
        const delay = config.enableDelay ? 100 * index : 0;
        setTimeout(() => {
            lineElement.classList.add("visible");
            resolve();
        }, delay);
    });
}

/**
 * Animates the loading progress bar
 * @param {number} duration - Duration of the animation in milliseconds
 * @returns {Promise<void>}
 */
function animateProgressBar(duration) {
    return new Promise((resolve) => {
        const startTime = Date.now();
        const intervalId = setInterval(() => {
            const elapsed = Date.now() - startTime;
            const progress = Math.min(elapsed / duration, 1);

            loadingBar.style.width = `${progress * 100}%`;

            if (progress >= 1) {
                clearInterval(intervalId);
                resolve();
            }
        }, 30);
    });
}

/**
 * Ensures the loading screen is initialized
 * Creates it immediately if not already done
 */
function ensureInitialized() {
    if (!isInitialized) {
        createLoadingScreen();
    }
}

/**
 * Shows the loading screen
 * @param {string} destination - The URL to navigate to
 * @returns {Promise<void>}
 */
async function showLoadingScreen(destination) {
    // Ensure the loading screen DOM elements exist
    ensureInitialized();

    // Select one random ASCII art for this load
    const randomIndex = Math.floor(
        Math.random() * config.loadingMessages.length,
    );
    const currentMessages = [config.loadingMessages[randomIndex]];

    // Clear previous content
    loadingTextContainer.innerHTML = "";
    loadingBar.style.width = "0%";

    // Show the loading screen
    loadingScreen.classList.add("active");

    // Display messages with minimal or no delay between them
    for (let i = 0; i < currentMessages.length; i++) {
        await showLoadingMessage(currentMessages[i], i);

        // Only add message delay if enabled
        if (config.enableDelay) {
            await new Promise((resolve) => setTimeout(resolve, 200));
        }
    }

    // Animate progress bar - choose duration based on config
    const duration = config.enableDelay
        ? Math.random() * 700 + 800 // 800-1500ms with delay enabled
        : config.minAnimationTime; // Minimal delay when disabled

    await animateProgressBar(duration);

    // Navigate to the destination
    window.location.href = destination;
}

/**
 * Navigate to a URL with the loading screen
 * @param {string} destination - The URL to navigate to
 */
export async function navigateWithLoading(destination) {
    // Don't show loading screen for hash changes
    if (
        destination.startsWith("#") ||
        destination === window.location.href ||
        destination === window.location.href + "#"
    ) {
        window.location.href = destination;
        return;
    }

    // Show the loading screen
    await showLoadingScreen(destination);

    // The actual navigation will happen at the end of showLoadingScreen
}

/**
 * Initializes the loading screen
 */
export function initLoadingScreen() {
    // Wait for styles to be fully loaded
    if (document.readyState === "complete") {
        createLoadingScreen();
    } else {
        window.addEventListener("load", createLoadingScreen);
    }
}

/**
 * Initializes the loading screen when DOM is loaded
 */
document.addEventListener("DOMContentLoaded", initLoadingScreen);

/**
 * Configure the loading screen
 * @param {Object} options - Configuration options
 * @param {boolean} [options.enableDelay] - Enable or disable loading screen delay
 * @param {number} [options.minAnimationTime] - Minimum animation time in milliseconds
 * @param {number} [options.maxAnimationTime] - Maximum animation time in milliseconds
 */
export function configureLoadingScreen(options = {}) {
    if (typeof options.enableDelay === "boolean") {
        config.enableDelay = options.enableDelay;
    }
    if (typeof options.minAnimationTime === "number") {
        config.minAnimationTime = options.minAnimationTime;
    }
    if (typeof options.maxAnimationTime === "number") {
        config.maxAnimationTime = options.maxAnimationTime;
    }
}

export default {
    initLoadingScreen,
    navigateWithLoading,
    configureLoadingScreen,
};
