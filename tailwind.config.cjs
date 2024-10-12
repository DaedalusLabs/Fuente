/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ['**/*.{html,rs}', '../**/*.{html,rs}'],
    plugins: [
        require('@tailwindcss/forms'),
        require('@tailwindcss/typography'),
    ],
    theme: {
        extend: {
            colors: {
                'fuente-light': '#2aa1e2',
                'fuente': '#4167e8',
                'fuente-dark': '#3b1197',

            },
            fontFamily: {
                'product': ['OpenSans', 'sans-serif'],
                'mplus': ['MPlus', 'sans-serif'],
            },
        },
    },
};
