# Frontend for static Early Adopter website

## 🚀 Project Structure

```text
/
├── public/
│   └── static files like images and fonts
├── src/
│   ├── components/
│   │   └── Card.astro
│   ├── layouts/
│   │   └── Layout.astro
│   ├── pages/
│   │   └── index.astro
│   └── contents/
│       └── index.json
```

Astro looks for `.astro` or `.md` files in the `src/pages/` directory. Each page is exposed as a route based on its file name.

There's nothing special about `src/components/`, but that's where we like to put any Astro/React/Vue/Svelte/Preact components.

Any static assets, like images, can be placed in the `public/` directory.

## Development

### Environment Variables

Create a `.env` file in the same directory as this README and set the env var `PUBLIC_INTERNET_IDENTITY_URL` to your development identity provider.

In production, it defaults to `https://identity.ic0.app/`.

### 🧞 Commands

All commands are run from the root of the project, from a terminal:

| Command                   | Action                                           |
| :------------------------ | :----------------------------------------------- |
| `npm install`             | Installs dependencies                            |
| `npm run dev`             | Starts local dev server at `localhost:4321`      |
| `npm run build`           | Build your production site to `./dist/`          |
| `npm run preview`         | Preview your build locally, before deploying     |
| `npm run astro ...`       | Run CLI commands like `astro add`, `astro check` |
| `npm run astro -- --help` | Get help using the Astro CLI                     |

## 📚 Astro

This project was bootstrapped with [Astro](https://astro.build/). For more information, check out the [Astro documentation](https://docs.astro.build/).
