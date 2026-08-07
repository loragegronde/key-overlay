/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        mono: ["JetBrains Mono", "Fira Code", "monospace"],
        display: ["Orbitron", "sans-serif"],
      },
      // No custom keyframes: every press animation is a framer-motion target
      // now. Keeping them here invited runtime-built class names like
      // `animate-${effect}`, which the scanner cannot see and therefore purges.
    },
  },
  plugins: [],
};
