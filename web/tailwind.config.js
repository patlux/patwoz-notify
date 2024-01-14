/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        brand: {
          50: '#F0F0EF',
          100: '#E4E4E2',
          200: '#C7C6C1',
          300: '#ADAAA4',
          400: '#908C84',
          500: '#737068',
          600: '#5B5952',
          700: '#46443F',
          800: '#2E2C29',
          900: '#181716',
        },
        accent: {
          50: '#FBFCF8',
          100: '#F5F7EE',
          200: '#ECEEDD',
          300: '#E4E8CF',
          400: '#DADFBE',
          500: '#D1D7AD',
          600: '#B3BE79',
          700: '#929E4D',
          800: '#5F6732',
          900: '#2F3319',
        },
      },
    },
  },
  plugins: [],
}
