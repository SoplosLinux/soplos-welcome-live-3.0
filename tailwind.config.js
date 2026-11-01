/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/**/*.{html,js,svelte,ts}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: "#ff7d2e",
          hover: "#e66a1f",
        },
        secondary: "#fec431",
        accent: "#ef5a33",
        dark: "rgb(var(--color-dark) / <alpha-value>)",
        panel: "rgb(var(--color-panel) / <alpha-value>)",
        surface: {
          DEFAULT: "rgb(var(--color-surface) / <alpha-value>)",
          2: "rgb(var(--color-surface-2) / <alpha-value>)",
        },
        border: "rgb(var(--color-border) / <alpha-value>)",
        text: {
          DEFAULT: "rgb(var(--color-text) / <alpha-value>)",
          bright: "rgb(var(--color-text-bright) / <alpha-value>)",
          muted: "rgb(var(--color-text-muted) / <alpha-value>)",
        },
        success: "#22c55e",
        warning: "#f59e0b",
        danger: "#ef4444",
      },
      borderRadius: {
        DEFAULT: "8px",
      },
    },
  },
  plugins: [],
};
