/**
 * Lighthouse-compatible throttling profiles
 *
 * These profiles match PageSpeed Insights and Lighthouse lab conditions
 * for accurate and comparable performance measurements.
 *
 * @see https://github.com/GoogleChrome/lighthouse/blob/main/core/config/constants.js
 */

export interface ThrottlingProfile {
  name: string;
  description: string;
  network: {
    latencyMs: number;
    downloadKbps: number;
    uploadKbps: number;
  };
  cpu: {
    slowdownMultiplier: number;
  };
}

/**
 * Lighthouse "Slow 4G" profile (Mobile - DEFAULT)
 *
 * This is the default mobile profile used by PageSpeed Insights.
 * Simulates a mobile device on a slow 4G connection.
 *
 * Network: 400ms RTT, 400 Kbps download, 400 Kbps upload
 * CPU: 4x slowdown
 */
export const SLOW_4G: ThrottlingProfile = {
  name: 'slow-4g',
  description: 'Lighthouse Slow 4G (Mobile - Default)',
  network: {
    latencyMs: 400,     // Round-trip time
    downloadKbps: 400,  // Download throughput
    uploadKbps: 400,    // Upload throughput
  },
  cpu: {
    slowdownMultiplier: 4, // CPU throttling
  },
};

/**
 * Lighthouse "Fast 3G" profile (Mobile - Optimistic)
 *
 * Simulates a mobile device on a faster 3G connection.
 * More optimistic than Slow 4G, but still realistic for mobile.
 *
 * Network: 562.5ms RTT, 1600 Kbps download, 750 Kbps upload
 * CPU: 4x slowdown
 */
export const FAST_3G: ThrottlingProfile = {
  name: 'fast-3g',
  description: 'Lighthouse Fast 3G (Mobile - Optimistic)',
  network: {
    latencyMs: 562.5,   // Slower latency than Slow 4G
    downloadKbps: 1600, // Faster download
    uploadKbps: 750,    // Faster upload
  },
  cpu: {
    slowdownMultiplier: 4,
  },
};

/**
 * Desktop profile (No Throttling)
 *
 * Desktop testing without network or CPU throttling.
 * Use this for desktop-specific performance testing.
 *
 * Network: No throttling
 * CPU: No throttling
 */
export const DESKTOP: ThrottlingProfile = {
  name: 'desktop',
  description: 'Desktop (No Throttling)',
  network: {
    latencyMs: 0,
    downloadKbps: 0, // 0 means no throttling
    uploadKbps: 0,
  },
  cpu: {
    slowdownMultiplier: 1, // No CPU throttling
  },
};

/**
 * 3G Regular profile
 *
 * Standard 3G network conditions.
 *
 * Network: 300ms RTT, 750 Kbps download, 250 Kbps upload
 * CPU: 4x slowdown
 */
export const REGULAR_3G: ThrottlingProfile = {
  name: '3g',
  description: '3G Regular',
  network: {
    latencyMs: 300,
    downloadKbps: 750,
    uploadKbps: 250,
  },
  cpu: {
    slowdownMultiplier: 4,
  },
};

/**
 * 2G profile (Very Slow)
 *
 * Simulates very slow network conditions (2G).
 *
 * Network: 800ms RTT, 280 Kbps download, 256 Kbps upload
 * CPU: 4x slowdown
 */
export const SLOW_2G: ThrottlingProfile = {
  name: '2g',
  description: '2G (Very Slow)',
  network: {
    latencyMs: 800,
    downloadKbps: 280,
    uploadKbps: 256,
  },
  cpu: {
    slowdownMultiplier: 4,
  },
};

/**
 * All available throttling profiles
 */
export const THROTTLING_PROFILES: Record<string, ThrottlingProfile> = {
  'slow-4g': SLOW_4G,
  'fast-3g': FAST_3G,
  'desktop': DESKTOP,
  '3g': REGULAR_3G,
  '2g': SLOW_2G,
};

/**
 * Default throttling profile (Lighthouse Slow 4G)
 */
export const DEFAULT_PROFILE = SLOW_4G;

/**
 * Get throttling profile by name
 */
export function getThrottlingProfile(name: string): ThrottlingProfile {
  const profile = THROTTLING_PROFILES[name.toLowerCase()];
  if (!profile) {
    console.warn(`Unknown throttling profile: ${name}, using default (slow-4g)`);
    return DEFAULT_PROFILE;
  }
  return profile;
}

/**
 * Format throttling profile for display
 */
export function formatThrottlingProfile(profile: ThrottlingProfile): string {
  if (profile.network.downloadKbps === 0) {
    return 'No Throttling (Desktop)';
  }
  return `${profile.description} · RTT ${profile.network.latencyMs}ms · ${profile.network.downloadKbps}kbps down · ${profile.network.uploadKbps}kbps up · CPU×${profile.cpu.slowdownMultiplier}`;
}
