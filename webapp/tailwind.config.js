/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./webapp/**/*.{html,js}",
    "./plugins/**/*.{html,js}"
  ],
  theme: {
    extend: {},
  },
  plugins: [require("daisyui")],
  daisyui: {
    themes: true, // This enables all daisyUI themes
    darkTheme: "dark",
  },
}
