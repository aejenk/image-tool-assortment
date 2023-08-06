# Image Tool Assortment

Recently I made a repository called [image-effects](https://github.com/enbyss/image-filters) which effectively hosts a custom-made library for applying multiple effects to images, such as:

- dithering (error-propagation *(multiple algorithms)* **and** ordered *(bayer)*)
- brightness (luminance manipulation via *LCH*)
- gradient mapping (map each pixel's *luminance* to a colour)
- hue rotation (hue manipulation via *LCH*)

Feel free to browse that repo! I don't have much (if any) experience as a library designer, so things may be a bit of a mess - but if you want to open a pull request to improve something, go ahead! I'd love to see it fluorish.

**This repository** is a *Cargo workspace* designed to host an assortment of packages that utilize the `image-filters` library in multiple ways. **Please note** that `image-filters` library is still experimental and unstable, and may change significantly - keep that in mind before writing your project.

## Packages

### `nasa-apod-generator`

Automatically retrieves an image using NASA's APOD API, then dithers it according to a random palette. Requires an api key be set in `.env` *(of **this** directory, not `nasa-apod-generator/.env`)*. 

Can definitely be useful for creating an automated bot that posts some nice looking images every now and again. Infact, I'm planning on doing just that at some point.

### `basic`

Basic usage of the library. In other words, like someone pulled the library from github and started playing with it to generate images. If you've seen me post dithered images, it's likely because I was playing with *this*.

## Plans

### `animated-dither`

Effectively allows for dithering *gifs* and potentially *videos*. That should be possible from what I've seen - there's crates for **both** at least. However since the library currently only works with `DynamicImage`, an update to that might need to happen first.

### `imgtoy`

A **CLI**... or **TUI**... or **GUI**... of the library to serve as a good interface. Purpose of this being to provide a way for non-technical people *(or even technical people who don't wanna bother)* a way to easily apply their own effects on an image. This *feels* like it'd be some effort, and requires I know how to make a GUI.

So far **Tauri** seems like my preferred pick, since I'm very good with Svelte. I don't want to muck much with the lower levels... at least not yet *(or at all, Tauri feels like it should be good enough)*.