const defaultTheme = require("tailwindcss/defaultTheme");

module.exports = {
    content: [
        "./src/**/*.{rs,html,css}",
        "./index.html",
        "./styles/**/*.css",
    ],
    theme: {
        screens: {
            "2xs": "370px",
            xs: "475px",
            ...defaultTheme.screens,
            "3xl": "1792px",
        },
        extend: {
            colors: {},
        },
    },
    variants: {},
};
