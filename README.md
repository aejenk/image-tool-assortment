# Image Tool Assortment

Recently I made a repository called [image-effects](https://github.com/enbyss/image-filters) which effectively hosts a custom-made library for applying effects to images.

These effects include anything from *dithering*, *brightness*, *gradient mapping*, *hue rotation* and more.

**This repository** is a *Cargo workspace* designed to host an assortment of packages that utilize the `image-filters` library in multiple ways. **Please note** that `image-filters` library is still experimental and unstable, and may change significantly - keep that in mind before writing your project.

Currently the packages involved are:

- **`nasa-apod-generator`**: Automatically retrieves an image using NASA's APOD API, then dithers it according to a random palette. Requires an api key be set in `.env` *(of **this** directory, not `nasa-apod-generator/.env`)*
- **`basic`**: Basic usage of the library. In other words, like someone pulled the library from github and started playing with it to generate images.