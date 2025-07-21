import { defineConfig } from "@chakra-ui/react";

// Neumorphic color palette
const neumorphicColors = {
  light: {
    bg: '#e0e5ec',
    surface: '#e0e5ec',
    text: '#2d3748',
    textSecondary: '#4a5568',
    accent: '#3182ce',
    shadow: {
      dark: '#a3b1c6',
      light: '#ffffff',
    },
  },
  dark: {
    bg: '#2d3748',
    surface: '#ffffff',
    text: '#f7fafc',
    textSecondary: '#e2e8f0',
    accent: '#63b3ed',
    shadow: {
      dark: '#1a202c',
      light: '#4a5568',
    },
  },
};

// Neumorphic shadow mixins
const neumorphicShadows = {
  light: {
    raised: '1px 1px 2px #a3b1c6, -1px -1px 2px #ffffff',
    pressed: 'inset 2px 2px 4px #a3b1c6, inset -2px -2px 4px #ffffff',
    subtle: '1px 1px 2px #a3b1c6, -1px -1px 2px #ffffff',
    subtlePressed: 'inset 1px 1px 2px #a3b1c6, inset -1px -1px 2px #ffffff',
    floating: '6px 6px 12px #a3b1c6, -6px -6px 12px #ffffff',
  },
  dark: {
    raised: '2px 2px 4px #1a202c, -2px -2px 4px #4a5568',
    pressed: 'inset 2px 2px 4px #1a202c, inset -2px -2px 4px #4a5568',
    subtle: '2px 2px 2px #1a202c, -2px -2px 2px #4a5568',
    subtlePressed: 'inset 2px 2px 2px #1a202c, inset -2px -2px 2px #4a5568',
    floating: '6px 6px 12px #1a202c, -6px -6px 12px #4a5568',
  },
};

// Simplified theme configuration to avoid TypeScript errors
// The utility functions below provide the neumorphic styling functionality
export const neumorphicTheme = defineConfig({
  theme: {
    // Theme configuration simplified to avoid type errors
  },
});

// Utility functions for neumorphic styling
export const getNeumorphicStyle = (colorMode: 'light' | 'dark', variant: 'raised' | 'pressed' | 'subtle' | 'floating' = 'raised') => {
  const colors = neumorphicColors[colorMode];
  const shadows = neumorphicShadows[colorMode];
  
  return {
    bg: colors.surface,
    color: colors.text,
    borderRadius: 6,
    boxShadow: shadows[variant] || shadows.raised,
    transition: 'all 0.3s ease-in-out',
  };
};

export const getNeumorphicColors = (colorMode: 'light' | 'dark') => {
  return neumorphicColors[colorMode];
};

export const getNeumorphicShadows = (colorMode: 'light' | 'dark') => {
  return neumorphicShadows[colorMode];
};