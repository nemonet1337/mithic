/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/**/*.rs"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        paper: "var(--paper)",
        ink: "var(--ink)",
        muted: "var(--ink-3)",
        line: "var(--line)",
        card: "var(--card)",
        accent: "var(--accent)",
      },
      fontFamily: {
        hand: ["var(--font-hand)"],
        body: ["var(--font-body)"],
        mono: ["var(--font-mono)"],
      },
      boxShadow: {
        sketch: "var(--shadow)",
      },
    },
  },
  plugins: [],
};
