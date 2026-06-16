/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/**/*.rs"],
  darkMode: ["class", '[data-theme="night"]'],
  theme: {
    extend: {},
  },
  plugins: [require("daisyui")],
  daisyui: {
    themes: ["light", "night"],
    darkTheme: "night",
    base: true,
    styled: true,
    utils: true,
    logs: false,
  },
};
