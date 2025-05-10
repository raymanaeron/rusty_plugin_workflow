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
    themes: ["light", "dark", "cupcake", "corporate", "emerald"],
    darkTheme: "dark",
  },
}
