<!-- This file is rendered at build time into HTML and included in the binary as the help page. -->

## Using Evanescence

For the best experience, please view this page on a computer.

### Visualization modes

You can select different visualization modes using the tab bar:

![Tab bar](img/tab-bar.png)

Each mode offers a different type of orbital to visualize.

* "Real (Simple)" provides a selection of common atomic orbitals for quick access.
* "Real (Full)" allows arbitrary quantum numbers (up to *n* = 8) to be specified.
* "Complex" displays the complex orbitals, which are useful in certain contexts.
* "Hybrid" provides visualizations for hybridized orbitals and their symmetry.

### Help for specific features

Text with <span class="tooltip">dotted underlines<div class="description"><div><p>I'm a tooltip!</p></div></div></span> have explanations. Hover over them with your cursor to read them.

#### 3D isosurface visualization is blank

If you see a blank isosurface plot, try increasing the quality – the orbital may be too intricate to be sufficiently well sampled at the current quality.

### Interacting with visualizations

For 3D visualizations, you can:

* Left-click and drag to rotate
* Right-click and drag to pan
* Scroll to zoom

Additional options are available in the toolbar that appears when you hover over a plot. For example, you can use the "camera" tool to save a screenshot of the plot.

## About Evanescence

&copy; Alvin Meng 2021

Built in [Rust](https://rust-lang.org) and WebAssembly using the [Yew](https://yew.rs) framework and the [Plotly.js](https://plotly.com/javascript) graphing library.

This software is released under the [GNU Affero GPL v3](https://www.gnu.org/licenses/agpl-3.0.en.html) license. The source code can be found on [GitHub](https://github.com/al2me6/evanescence).
