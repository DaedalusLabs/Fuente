/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ['**/*.{html,rs}', '../**/*.{html,rs}', '!/node_modules/**', '!./**/target/**'],
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
                'fuente-forms': "#3859c7",
                "fuente-buttons": "#6fe5fa",
                "fuente-orange": "#f4801b"
            },
            fontFamily: {
                'mplus': ['MPlus', 'sans-serif'],
                'fuente': ['Open Sans'],
                'product': ['ProductSans', 'sans-serif']
            },
            backgroundImage: {
                logo: `url('/templates/img/Logo Fuente.jpeg')`
            }
        },
    },
};
